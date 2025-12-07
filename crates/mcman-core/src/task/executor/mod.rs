use crate::task::Task;

trait TaskExecutor {
    async fn execute<T: Task>(&self, task: T) -> miette::Result<T::Output>;
}
