//! Host-only ABI probe. This callback is not a PocketJS/UEFI implementation.
use core::ffi::c_void;
use core::mem::size_of;
use pocketjs_boot_context_abi as abi;

unsafe extern "C" {
    fn pjb_test_call(
        table: *const abi::ContextTable,
        request: *const abi::Request,
        response: *mut abi::Response,
    ) -> abi::Status;
    fn pjb_test_invoke(
        table: *const abi::ContextTable,
        request: *const abi::Request,
        response: *mut abi::Response,
    ) -> abi::Status;
    fn pjb_test_guid(output: *mut abi::Guid);
}

struct Probe {
    calls: usize,
    result: abi::Status,
    consumer: *mut c_void,
}

unsafe extern "efiapi" fn owner_entry(
    instance: *mut c_void,
    request: *const abi::Request,
    response: *mut abi::Response,
) -> abi::Status {
    let probe = unsafe { &mut *instance.cast::<Probe>() };
    let request = unsafe { &*request };
    let response = unsafe { &mut *response };
    probe.calls += 1;
    if request.operation != abi::OP_PUMP || request.lease != 0x1020_3040_5060_7080
        || request.consumer_image_handle != probe.consumer {
        return abi::UNSUPPORTED;
    }
    response.lease = request.lease;
    probe.result
}

fn main() {
    let mut probe = Probe {
        calls: 0,
        result: abi::SUCCESS,
        consumer: core::ptr::null_mut(),
    };
    let mut image_marker = 0_u8;
    let image = core::ptr::addr_of_mut!(image_marker).cast::<c_void>();
    probe.consumer = image;
    let mut child_image_marker = 0_u8;
    let child_image = core::ptr::addr_of_mut!(child_image_marker).cast::<c_void>();
    let heap = [0_u8; 256];
    let range = abi::HeapRange {
        base_address: heap.as_ptr() as u64,
        byte_length: heap.len() as u64,
        kind: abi::HEAP_MIXED,
        reserved: 0,
    };
    let mut table = abi::ContextTable {
        signature: abi::SIGNATURE,
        struct_size: size_of::<abi::ContextTable>() as u32,
        abi_major: abi::ABI_MAJOR,
        abi_minor: abi::ABI_MINOR,
        machine_type: abi::MACHINE,
        pointer_bits: 64,
        capabilities: abi::REQUIRED_CAPABILITIES,
        generation: 9,
        owner_image_handle: image,
        runtime_instance: core::ptr::addr_of_mut!(probe).cast::<c_void>(),
        heap_ranges: &range,
        heap_range_count: 1,
        heap_range_entry_size: size_of::<abi::HeapRange>() as u32,
        invoke: Some(owner_entry),
        reserved: [0; 4],
    };
    let mut request = abi::Request {
        struct_size: size_of::<abi::Request>() as u32,
        operation: abi::OP_PUMP,
        generation: table.generation,
        consumer_image_handle: image,
        lease: 0x1020_3040_5060_7080,
    };
    let mut response = abi::Response {
        struct_size: size_of::<abi::Response>() as u32,
        reserved: 0,
        lease: 0,
    };

    assert_eq!(unsafe { pjb_test_call(&table, &request, &mut response) }, abi::SUCCESS);
    assert_eq!(probe.calls, 1);
    assert_eq!(response.lease, request.lease);
    assert_eq!(unsafe { pjb_test_invoke(&table, &request, &mut response) }, abi::SUCCESS);
    assert_eq!(probe.calls, 2);
    let mut guid = abi::Guid { data1: 0, data2: 0, data3: 0, data4: [0; 8] };
    unsafe { pjb_test_guid(&mut guid) };
    assert_eq!(guid, abi::CONTEXT_GUID);

    // Forward distinct parent/child image handles while preserving the owner,
    // runtime address, heap descriptor, generation and lease. This is an ABI
    // forwarding probe, not LoadImage/StartImage or a lease-service implementation.
    let original_instance = table.runtime_instance;
    for _ in 0..32 {
        for consumer in [child_image, image] {
            probe.consumer = consumer; request.consumer_image_handle = consumer;
            assert_eq!(unsafe { pjb_test_call(&table, &request, &mut response) }, abi::SUCCESS);
            assert_eq!(table.runtime_instance, original_instance);
            assert_eq!(table.owner_image_handle, image);
            assert!(core::ptr::eq(table.heap_ranges, &range));
            assert_eq!(response.lease, request.lease);
        }
    }
    assert_eq!(probe.calls, 66);
    println!("PASS: 32 parent/child handle forwarding cycles; stable owner/runtime/heap/generation/lease");

    // A full-width EFI error must survive both calling conventions unchanged.
    probe.result = abi::INVALID_PARAMETER;
    assert_eq!(
        unsafe { pjb_test_call(&table, &request, &mut response) },
        abi::INVALID_PARAMETER
    );
    assert_eq!(probe.calls, 67);

    // A future incompatible table must not be invoked by the C consumer.
    table.abi_major += 1;
    assert_eq!(unsafe { pjb_test_call(&table, &request, &mut response) }, abi::UNSUPPORTED);
    assert_eq!(probe.calls, 67);
    assert_eq!(
        unsafe { pjb_test_call(core::ptr::null(), &request, &mut response) },
        abi::INVALID_PARAMETER
    );
    println!("PASS: generated C header -> Rust dispatcher/efiapi callback; GUID, pointer, response, status, ABI rejection");
}
