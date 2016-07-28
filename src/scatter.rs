extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::task::{Task};
use super::connectable::{Connectable};

pub trait Scatter {
  type InputType   : Copy+Send;
  type OutputType  : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Message<Self::InputType>>,
    output:  &mut Vec<Sender<Message<Self::OutputType>>>) -> Schedule;
}

pub struct ScatterWrap<Input: Copy+Send, Output: Copy+Send> {
  name           : String,
  state          : Box<Scatter<InputType=Input,OutputType=Output>+Send>,
  input_rx       : Option<IdentifiedReceiver<Input>>,
  output_tx_vec  : Vec<Sender<Message<Output>>>,
}

impl<Input: Copy+Send, Output: Copy+Send> Connectable for ScatterWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Copy+Send, Output: Copy+Send> Task for ScatterWrap<Input,Output> {
  fn execute(&mut self) -> Schedule {
    match &mut self.input_rx {
      &mut Some(ref mut identified) => {
        self.state.process(
          &mut identified.input,
          &mut self.output_tx_vec
        )
      },
      &mut None => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Input: Copy+Send, Output: Copy+Send>(
    name            : &str,
    output_q_size   : usize,
    scatter         : Box<Scatter<InputType=Input,OutputType=Output>+Send>,
    n_channels      : usize)
      -> (Box<ScatterWrap<Input,Output>>, Vec<Box<Option<IdentifiedReceiver<Output>>>>)
{
  let mut tx_vec = vec![];
  let mut rx_vec = vec![];

  for i in 0..n_channels {
    let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
    tx_vec.push(output_tx);
    rx_vec.push(
      Box::new(
        Some(
          IdentifiedReceiver{
            id:     new_id(String::from(name), Direction::Out, i),
            input:  output_rx,
          }
        )
      )
    );
  }

  (
    Box::new(
      ScatterWrap{
        name           : String::from(name),
        state          : scatter,
        input_rx       : None,
        output_tx_vec  : tx_vec,
      }
    ),
    rx_vec
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
