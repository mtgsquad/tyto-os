use boot_lib::KernelArgs;

pub(crate) mod framebuffer;
pub(crate) mod framebuffer_term;

pub(crate) fn init(args: &mut KernelArgs) {
    framebuffer::init(args);
}
