mod command;
mod error;
mod runner;
mod state;

pub use command::*;
pub use error::*;
pub use runner::*;
pub use state::*;

#[cfg(feature = "derive")]
pub use derive::*;

#[cfg(test)]
mod test {
    use crate::*;
    use runner::{Commander, Printer, Runner};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Default)]
    struct EasyControlInfo {
        next_task: usize,
    }

    #[derive(Debug, Deserialize, Serialize, AbstractState)]
    struct EasyState {
        tasks: IdentList<usize>,
        #[serde(skip_serializing)]
        control: Ignored<EasyControlInfo>,
    }

    struct Spawn;

    impl Command<EasyState> for Spawn {
        fn execute(&self, state: &mut EasyState) -> Result<()> {
            state.tasks.0.push(state.control.0.next_task);
            state.control.0.next_task += 1;
            Ok(())
        }
        fn stringify(&self) -> String {
            "spawn".to_string()
        }
    }

    struct Sched;

    impl Command<EasyState> for Sched {
        fn execute(&self, state: &mut EasyState) -> Result<()> {
            let head = state.tasks.0[0];
            state.tasks.0.remove(0);
            state.tasks.0.push(head);
            Ok(())
        }
        fn stringify(&self) -> String {
            "sched".to_string()
        }
    }

    struct Exit;

    impl Command<EasyState> for Exit {
        fn execute(&self, state: &mut EasyState) -> Result<()> {
            state.tasks.0.pop();
            Ok(())
        }
        fn stringify(&self) -> String {
            "exit".to_string()
        }
    }

    struct RoundIn(usize);

    impl Commander<EasyState> for RoundIn {
        fn command(&mut self) -> Result<Box<dyn Command<EasyState>>> {
            let ops = vec![
                "spawn", "sched", "sched", "spawn", "sched", "exit", "sched", "spawn", "exit",
                "exit",
            ];
            let res = ops[(self.0) % ops.len()].to_string();
            self.0 += 1;
            match res.as_str() {
                "spawn" => Ok(Box::new(Spawn)),
                "sched" => Ok(Box::new(Sched)),
                "exit" => Ok(Box::new(Exit)),
                _ => Err(Error::new(ErrorKind::CommandNotFound)),
            }
        }
    }

    struct Stdout;

    impl Printer<EasyState> for Stdout {
        fn print_str(&mut self, s: &str) -> Result<()> {
            println!("{}", s);
            Ok(())
        }
        fn print_state(&mut self, s: &EasyState) -> Result<()> {
            let sta_str =
                serde_json::to_string(&s).map_err(|_| Error::new(ErrorKind::StateParseError))?;
            println!("{}", sta_str);
            Ok(())
        }
    }

    struct FakeTestPort(EasyState);

    impl TestPort<EasyState> for FakeTestPort {
        fn send(&mut self, command: &str) -> Result<()> {
            let command: Box<dyn Command<EasyState>> = match command {
                "spawn" => Box::new(Spawn),
                "sched" => Box::new(Sched),
                "exit" => Box::new(Exit),
                _ => return Err(Error::new(ErrorKind::CommandNotFound)),
            };
            command.execute(&mut self.0)
        }
        fn receive(&mut self) -> Result<&EasyState> {
            let sta_str = serde_json::to_string(&self.0)
                .map_err(|_| Error::new(ErrorKind::StateParseError))?;
            let _sta = serde_json::from_str::<EasyState>(&sta_str)
                .map_err(|_| Error::new(ErrorKind::StateParseError))?;
            Ok(&self.0)
        }
    }

    #[test]
    fn test_runner() {
        let state0 = EasyState {
            tasks: IdentList(vec![0]),
            control: Ignored(EasyControlInfo { next_task: 1 }),
        };
        let state1 = EasyState {
            tasks: IdentList(vec![100]),
            control: Ignored(EasyControlInfo { next_task: 101 }),
        };
        let mut runner = Runner::new(RoundIn(0), Stdout, FakeTestPort(state1), state0);
        for _ in 0..1000 {
            println!("=====================================");
            runner.step().expect("Runner Exited");
        }
    }
}
