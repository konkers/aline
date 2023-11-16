use core::mem::MaybeUninit;

use crate::{dyner, Result};

pub trait CommandExecutor {
    // TODO: add output context.
    async fn execute(&mut self) -> Result<()>;
}

type ExecuteFuture<'a> = dyner::InlineFuture<'a, Result<()>>;

pub trait InlineCommandExecutor {
    fn execute(&mut self) -> ExecuteFuture<'_>;
}

pub struct InlineDynCommandExecutor<'a> {
    obj: &'a mut dyn InlineCommandExecutor,
}

impl<'a> CommandExecutor for InlineDynCommandExecutor<'a> {
    async fn execute(&mut self) -> Result<()> {
        InlineCommandExecutor::execute(self.obj).await
    }
}

pub struct InlineCommandExecutorImpl<'a, I>
where
    I: CommandExecutor + 'a,
{
    inner_impl: I,
    execute_future: MaybeUninit<ExecuteFuture<'a>>,
}

impl<'a, I: CommandExecutor + 'a> InlineCommandExecutorImpl<'a, I> {
    pub fn new(inner_impl: I) -> Self {
        Self {
            inner_impl,
            execute_future: MaybeUninit::uninit(),
        }
    }

    pub fn as_dyn(&mut self) -> InlineDynCommandExecutor<'_> {
        InlineDynCommandExecutor { obj: self }
    }
}

impl<'a, I: CommandExecutor + 'a> InlineCommandExecutor for InlineCommandExecutorImpl<'a, I> {
    fn execute(&mut self) -> ExecuteFuture<'_> {
        let f = self.inner_impl.execute();

        // Deep magic here copied from dyner crate to "extend the lifetime of `f` artificially to `'a`.
        unsafe {
            let f_ptr = core::ptr::addr_of!(f) as *mut ExecuteFuture<'a>;
            self.execute_future.write(core::ptr::read(f_ptr));
            core::mem::forget(f);
        }

        unsafe { dyner::InlineFuture::new(&mut self.execute_future) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn my_test() {
        struct TestCommand1 {}
        impl CommandExecutor for TestCommand1 {
            async fn execute(&mut self) -> Result<()> {
                println!("cmd1");
                Ok(())
            }
        }
        struct TestCommand2 {}
        impl CommandExecutor for TestCommand2 {
            async fn execute(&mut self) -> Result<()> {
                println!("cmd2");
                Ok(())
            }
        }

        let mut inline_cmd1 = InlineCommandExecutorImpl::new(TestCommand1 {});
        let mut inline_dyn_cmd1 = inline_cmd1.as_dyn();
        let mut inline_cmd2 = InlineCommandExecutorImpl::new(TestCommand2 {});
        let inline_dyn_cmd2 = inline_cmd2.as_dyn();

        inline_dyn_cmd1.execute().await.unwrap();
        //let mut cmds = [inline_dyn_cmd1, inline_dyn_cmd2];

        // for cmd in &mut cmds {
        //     cmd.execute().await.unwrap();
        // }
    }
}
