use crate::{
    Config, Error, UserGeneration,
    budget::{self, Budget},
    ffi, memory,
    output::{DisplayId, DisplayOutput, Presentation, Renderer, RendererId},
    services,
};
use alloc::{borrow::Cow, boxed::Box, vec, vec::Vec};
use core::{
    ffi::{CStr, c_void},
    ptr::{self, NonNull},
};
use pocket_drivers::{InputFrame, abi::*};
use pocket_runtime::{CoreState, OpContext, Profile, RuntimeConfig, input::InputEncoder};
use pocket_runtime_cabi::embedding;

struct User {
    context: NonNull<ffi::JsContext>,
    guest: NonNull<ffi::Guest>,
    generation: UserGeneration,
    input: InputEncoder,
}

/// Own on one execution thread/core. This is deliberately !Send and !Sync.
/// The BSP chooses when to call tick; no scheduler, threads or device waits.
pub struct PocketJs {
    config: Config,
    engine: NonNull<ffi::JsRuntime>,
    runtime: NonNull<PocketRuntime>,
    ops: OpContext,
    budget: Box<Budget>,
    user: Option<User>,
    next_generation: u64,
    poisoned: bool,
    faulted: bool,
    mailbox: services::Shared,
    outputs: Presentation,
    debug: crate::debug::DebugChannel,
}

impl PocketJs {
    pub fn new(config: Config) -> Result<Self, Error> {
        config.validate()?;
        let outputs = Presentation::new(config.frame_buffer_budget)?;
        let runtime = embedding::create(RuntimeConfig {
            profile_hash: [0; 32],
            host_abi: pocket_spec::BINDING_GENERATION,
            heap_limit: config.js_heap_bytes,
            frame_buffer_budget: 0,
            profile: Profile {
                id: Cow::Borrowed("embedded"),
                tick_hz: config.tick_hz,
                viewport: [config.width, config.height],
                logical_viewports: Cow::Owned(vec![[config.width, config.height]]),
                dynamic_viewport: None,
                auxiliary: None,
                auxiliary_logical: Cow::Borrowed(&[]),
                density: 1,
                required_inputs: pocket_drivers::InputClasses::NONE,
            },
        })
        .map_err(|_| Error::Runtime)?;
        let runtime = NonNull::new(runtime).ok_or(Error::OutOfMemory)?;
        let ops = match unsafe { embedding::op_context(runtime.as_ptr()) } {
            Ok(ops) => ops,
            Err(_) => {
                unsafe { pocket_runtime_cabi::pocket_runtime_destroy(runtime.as_ptr()) };
                return Err(Error::Runtime);
            }
        };
        let mut budget = Budget::new();
        let engine = unsafe { ffi::JS_NewRuntime2(&memory::ALLOCATOR, ptr::null_mut()) };
        let Some(engine) = NonNull::new(engine) else {
            unsafe { pocket_runtime_cabi::pocket_runtime_destroy(runtime.as_ptr()) };
            return Err(Error::OutOfMemory);
        };
        unsafe {
            ffi::JS_SetMemoryLimit(engine.as_ptr(), config.js_heap_bytes);
            ffi::JS_SetMaxStackSize(engine.as_ptr(), config.js_stack_bytes);
            ffi::JS_SetInterruptHandler(
                engine.as_ptr(),
                budget::interrupt,
                (&mut *budget as *mut Budget).cast(),
            );
        }
        Ok(Self {
            config,
            engine,
            runtime,
            ops,
            budget,
            user: None,
            next_generation: 0,
            poisoned: false,
            faulted: false,
            mailbox: services::new(),
            outputs,
            debug: crate::debug::DebugChannel::default(),
        })
    }

    /// Stable initialized QuickJS runtime identity for same-address-space
    /// handoff. The pointer alone is not permission to access the VM or its heap.
    pub fn engine_handle(&self) -> NonNull<c_void> {
        self.engine.cast()
    }
    pub fn runtime_handle(&self) -> NonNull<PocketRuntime> {
        self.runtime
    }

    /// Opaque opt-in data only; the BSP supplies its transport and debugger.
    pub fn debug_channel(&mut self) -> &mut crate::debug::DebugChannel {
        &mut self.debug
    }

    /// Register a BSP accelerator implementing the official DrawList contract.
    /// The built-in software renderer is available as RendererId::SOFTWARE.
    pub fn add_renderer<R: Renderer + 'static>(&mut self, renderer: R) -> Result<RendererId, Error>
    where
        R::Submission: 'static,
    {
        self.complete_presentation()?;
        Ok(self.outputs.add_renderer(renderer)?)
    }

    /// Bind a BSP display/target to a renderer. Format, stride, buffers and
    /// synchronization are described by the drivers rather than VM config.
    pub fn add_display<D: DisplayOutput + 'static>(
        &mut self,
        display: D,
        renderer: RendererId,
    ) -> Result<DisplayId, Error>
    where
        D::Token: 'static,
    {
        self.complete_presentation()?;
        Ok(self.outputs.add_display(display, renderer)?)
    }

    /// Load trusted source scripts in order. All previous page globals, input
    /// captures, UI assets and service queues are discarded, retaining the VM.
    /// No package admission/signature checking is implied by this low-level API.
    pub fn replace_user(&mut self, scripts: &[&str]) -> Result<UserGeneration, Error> {
        self.replace_user_with(scripts, |_| Ok(()))
    }

    /// Load per-page atlases, textures or styles through the official Rust UI
    /// before mounting guest bindings. Static asset bytes may live in flash.
    pub fn replace_user_with(
        &mut self,
        scripts: &[&str],
        prepare: impl FnOnce(&mut pocketjs_core::Ui) -> Result<(), Error>,
    ) -> Result<UserGeneration, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        self.unload_user()?;
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or(Error::GenerationExhausted)?;
        {
            let mut core = self.ops.core_mut().map_err(|_| Error::Runtime)?;
            if let Err(error) = prepare(&mut core.ui) {
                drop(core);
                self.reset_ui()?;
                return Err(error);
            }
        }
        self.mailbox = services::new();
        self.ops
            .add_provider(Box::new(services::Provider(self.mailbox.clone())))
            .map_err(|_| Error::Runtime)?;
        self.arm();
        let context = NonNull::new(unsafe { ffi::JS_NewContext(self.engine.as_ptr()) })
            .ok_or(Error::OutOfMemory)?;
        let Some(guest) =
            NonNull::new(unsafe { ffi::pocket_guest_quickjs_attach(context.as_ptr(), 0) })
        else {
            unsafe { ffi::JS_FreeContext(context.as_ptr()) };
            return Err(Error::OutOfMemory);
        };
        let generation = UserGeneration(self.next_generation);
        self.user = Some(User {
            context,
            guest,
            generation,
            input: InputEncoder::new(),
        });
        self.faulted = false;
        let environment = PocketGuestEnvironment {
            host: b"embedded".as_ptr(),
            host_length: 8,
            host_abi: pocket_spec::BINDING_GENERATION,
            tick_hz: self.config.tick_hz,
            viewport_width: self.config.width,
            viewport_height: self.config.height,
            raster_density: 1,
            heap_limit: self.config.js_heap_bytes,
            ..PocketGuestEnvironment::empty()
        };
        let status = unsafe {
            self.table().initialize.unwrap()(
                guest.as_ptr().cast(),
                self.runtime.as_ptr(),
                &environment,
            )
        };
        let result = self.status(status).and_then(|()| {
            for source in scripts {
                self.evaluate(generation, source)?;
            }
            Ok(generation)
        });
        if result.is_err() {
            let _ = self.unload_user();
        }
        result
    }

    /// Refuse reuse if the runtime still owns jobs: freeing a context does not
    /// cancel Promise jobs. No old JS is executed as part of teardown.
    pub fn unload_user(&mut self) -> Result<(), Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        // Stop device reads before releasing guest-owned textures or nodes.
        self.complete_presentation()?;
        if unsafe { ffi::JS_IsJobPending(self.engine.as_ptr()) } {
            self.poisoned = true;
            return Err(Error::PendingJobs);
        }
        self.drop_user();
        unsafe { ffi::JS_RunGC(self.engine.as_ptr()) };
        self.reset_ui()?;
        self.mailbox = services::new();
        self.faulted = false;
        Ok(())
    }

    pub fn evaluate(&mut self, generation: UserGeneration, source: &str) -> Result<(), Error> {
        let guest = self.checked(generation)?.guest;
        self.arm();
        let status = unsafe {
            self.table().evaluate.unwrap()(
                guest.as_ptr().cast(),
                self.runtime.as_ptr(),
                source.as_ptr(),
                source.len(),
            )
        };
        self.status(status)
    }

    /// Prepare host assets after guest evaluation, keeping the page and its
    /// globals. Pending device reads finish before mutable resource access.
    pub fn prepare_resources<T>(
        &mut self,
        generation: UserGeneration,
        prepare: impl FnOnce(&mut pocketjs_core::Ui) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.checked(generation)?;
        self.complete_presentation()?;
        let mut core = self.ops.core_mut().map_err(|_| Error::Runtime)?;
        prepare(&mut core.ui)
    }

    /// Advance a fixed step and present. Immediate outputs complete here;
    /// Deferred accelerators can stay in flight through the next guest turn.
    pub fn tick(
        &mut self,
        generation: UserGeneration,
        input: &InputFrame<'_>,
    ) -> Result<(), Error> {
        self.advance(generation, input)?;
        self.present(generation)
    }

    /// Advance user input and UI without submitting a display frame. The BSP
    /// can schedule boot work or submit its native scene before present().
    pub fn advance(
        &mut self,
        generation: UserGeneration,
        input: &InputFrame<'_>,
    ) -> Result<(), Error> {
        self.checked(generation)?;
        let encoded = {
            let mut core = self.ops.core_mut().map_err(|_| Error::Runtime)?;
            self.user
                .as_mut()
                .unwrap()
                .input
                .encode(&mut core.ui, input)
                .map_err(|_| Error::InvalidInput)?
        };
        let relative: Vec<_> = input
            .relative
            .iter()
            .map(|a| PocketAxisDelta {
                axis: a.id,
                delta: a.delta,
                ..PocketAxisDelta::empty()
            })
            .collect();
        let text: Vec<_> = input
            .text
            .iter()
            .filter_map(crate::input::text_record)
            .collect();
        let motion = input.motion.map(crate::input::motion_header);
        let frame = PocketGuestFrame {
            buttons: encoded.buttons,
            analog: encoded.analog,
            right_analog: encoded.right_analog,
            touches: encoded.touches.as_ptr(),
            hits: encoded.hits.as_ptr(),
            touch_surfaces: encoded.surfaces.as_ptr(),
            touch_count: encoded.touches.len(),
            axis_deltas: relative.as_ptr(),
            axis_delta_count: relative.len(),
            motion: motion.as_ref().map_or(ptr::null(), |v| v),
            text: text.as_ptr(),
            text_count: text.len(),
            ..PocketGuestFrame::empty()
        };
        self.arm();
        let guest = self.user.as_ref().unwrap().guest;
        let status = unsafe { self.table().frame.unwrap()(guest.as_ptr().cast(), &frame) };
        self.status(status)?;
        self.ops.core_mut().map_err(|_| Error::Runtime)?.ui.tick();
        Ok(())
    }

    pub fn send(&mut self, generation: UserGeneration, message: &str) -> Result<(), Error> {
        self.checked(generation)?;
        services::send(&self.mailbox, message)
    }
    pub fn receive(&mut self, generation: UserGeneration) -> Result<Option<Vec<u8>>, Error> {
        self.checked(generation)?;
        Ok(services::receive(&self.mailbox))
    }
    /// Render through registered output drivers. With no displays this builds
    /// the DrawList only. Does not allocate an implicit CPU framebuffer.
    pub fn present(&mut self, generation: UserGeneration) -> Result<(), Error> {
        self.checked(generation)?;
        let mut core = self.ops.core_mut().map_err(|_| Error::Runtime)?;
        Ok(self.outputs.present(&mut core.ui)?)
    }

    /// Finish renderer/display work before handing off or accessing device
    /// buffers. Also callable after a script fault to stop pending hardware.
    pub fn complete_presentation(&mut self) -> Result<(), Error> {
        let mut core = self.ops.core_mut().map_err(|_| Error::Runtime)?;
        Ok(self.outputs.complete(&mut core.ui)?)
    }

    /// External scanout writes or host overlays invalidate retained damage.
    pub fn invalidate_outputs(&mut self) {
        self.outputs.invalidate();
    }

    fn checked(&self, generation: UserGeneration) -> Result<&User, Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let user = self.user.as_ref().ok_or(Error::NoUser)?;
        if user.generation != generation {
            return Err(Error::StaleGeneration);
        }
        if self.faulted {
            return Err(Error::Poisoned);
        }
        Ok(user)
    }
    fn table(&self) -> &'static PocketGuestV1 {
        unsafe { &*ffi::pocket_guest_quickjs_vtable() }
    }
    fn arm(&self) {
        self.budget.arm(self.config.interrupt_polls);
        unsafe { ffi::JS_UpdateStackTop(self.engine.as_ptr()) };
    }
    fn status(&mut self, status: i32) -> Result<(), Error> {
        if self.budget.exhausted.get() {
            self.faulted = true;
            return Err(Error::BudgetExceeded);
        }
        if status == 0 {
            return Ok(());
        }
        self.faulted = true;
        let message = self
            .user
            .as_ref()
            .map(|u| {
                unsafe { CStr::from_ptr(ffi::pocket_guest_quickjs_error(u.guest.as_ptr())) }
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or_default();
        Err(Error::Script(message))
    }
    fn reset_ui(&mut self) -> Result<(), Error> {
        let mut ui = pocketjs_core::Ui::new();
        pj4edk2_rendering::configure(&mut ui);
        ui.set_viewport(self.config.width as f32, self.config.height as f32);
        ui.set_tick_rate(self.config.tick_hz);
        *self.ops.core_mut().map_err(|_| Error::Runtime)? = CoreState::new(ui);
        self.outputs.invalidate();
        Ok(())
    }
    fn drop_user(&mut self) {
        if let Some(user) = self.user.take() {
            unsafe {
                ffi::pocket_guest_quickjs_destroy(user.guest.as_ptr());
                ffi::JS_FreeContext(user.context.as_ptr());
            }
        }
    }
}
impl Drop for PocketJs {
    fn drop(&mut self) {
        // Drivers guarantee terminal waits on both success and failure.
        let _ = self.complete_presentation();
        self.drop_user();
        // FreeRuntime discards queued jobs, after guest tokens were invalidated.
        // Budget and canonical op state remain alive throughout VM destruction.
        unsafe {
            ffi::JS_FreeRuntime(self.engine.as_ptr());
            pocket_runtime_cabi::pocket_runtime_destroy(self.runtime.as_ptr());
        }
    }
}
