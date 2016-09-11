use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, ChannelWrapper, ChannelId,
  SenderName, SenderChannelId, ReceiverChannelId, ReceiverName, ChannelPosition
};
use super::connectable::{ConnectableN};
use super::identified_input::{IdentifiedInput};
use super::counter::{OutputCounter, InputCounter};

pub trait Gather {
  type InputType   : Send;
  type OutputType  : Send;

  fn process(
    &mut self,
    input:   &mut Vec<ChannelWrapper<Self::InputType>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct GatherWrap<Input: Send, Output: Send> {
  name           : String,
  state          : Box<Gather<InputType=Input,OutputType=Output>+Send>,
  input_rx_vec   : Vec<ChannelWrapper<Input>>,
  output_tx      : Sender<Message<Output>>,
}

impl<Input: Send, Output: Send> IdentifiedInput for GatherWrap<Input,Output> {
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 < self.input_rx_vec.len() {
      let slice = self.input_rx_vec.as_slice();
      match &slice[ch_id.0] {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None,
      }
    } else {
      None
    }
  }
}

impl<Input: Send, Output: Send> InputCounter for GatherWrap<Input,Output> {
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 < self.input_rx_vec.len() {
      let slice = self.input_rx_vec.as_slice();
      match &slice[ch_id.0] {
        &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) => {
          receiver.seqno()
        },
        _ => 0,
      }
    } else {
      0
    }
  }
}

impl<Input: Send, Output: Send> OutputCounter for GatherWrap<Input,Output> {
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<Input: Send, Output: Send> ConnectableN for GatherWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self, n: ReceiverChannelId) -> &mut ChannelWrapper<Self::Input> {
    let ret_slice = self.input_rx_vec.as_mut_slice();
    &mut ret_slice[n.0]
  }
}

impl<Input: Send, Output: Send> Task for GatherWrap<Input,Output> {
  fn execute(&mut self) -> Schedule {
    self.state.process(&mut self.input_rx_vec, &mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { self.input_rx_vec.len() }
  fn output_count(&self) -> usize { 1 }

  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    self.get_input_id(ch_id)
  }

  fn input_channel_pos(&self, ch_id: ReceiverChannelId) -> ChannelPosition {
    ChannelPosition( self.get_rx_count(ch_id) )
  }

  fn output_channel_pos(&self, ch_id: SenderChannelId) -> ChannelPosition {
    ChannelPosition( self.get_tx_count(ch_id) )
  }
}

pub fn new<Input: Send, Output: Send>(
    name            : &str,
    output_q_size   : usize,
    gather          : Box<Gather<InputType=Input,OutputType=Output>+Send>,
    n_channels      : usize)
      -> (Box<GatherWrap<Input,Output>>, Box<ChannelWrapper<Output>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);
  let mut inputs = vec![];
  for i in 0..n_channels {
    inputs.push(ChannelWrapper::ReceiverNotConnected(
      ReceiverChannelId(i),
      ReceiverName (name.clone())
    ));
  }

  (
    Box::new(
      GatherWrap{
        name                   : name.clone(),
        state                  : gather,
        input_rx_vec           : inputs,
        output_tx              : output_tx,
      }
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_rx,
        SenderName(name)
      )
    )
  )
}
