use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, ChannelWrapper, ChannelId, SenderName,
  ReceiverChannelId, ReceiverName, SenderChannelId, ChannelPosition
};
use super::connectable::{ConnectableY};
use super::identified_input::{IdentifiedInput};
use super::counter::{OutputCounter, InputCounter};

pub trait YMerge {
  type InputTypeA   : Send;
  type InputTypeB   : Send;
  type OutputType   : Send;

  fn process(
    &mut self,
    input_a:  &mut ChannelWrapper<Self::InputTypeA>,
    input_b:  &mut ChannelWrapper<Self::InputTypeB>,
    output:   &mut Sender<Message<Self::OutputType>>) -> Result<(), &'static str>;
}

pub struct YMergeWrap<InputA: Send, InputB: Send, Output: Send> {
  name         : String,
  state        : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>+Send>,
  input_a_rx   : ChannelWrapper<InputA>,
  input_b_rx   : ChannelWrapper<InputB>,
  output_tx    : Sender<Message<Output>>,
}

impl<InputA: Send, InputB: Send, Output: Send> IdentifiedInput for YMergeWrap<InputA, InputB, Output> {
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 > 1 {
      None
    } else if ch_id.0 == 0 {
      match &self.input_a_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None
      }
    } else {
      match &self.input_b_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None
      }
    }
  }
}

impl<InputA: Send, InputB: Send, Output: Send> InputCounter for YMergeWrap<InputA, InputB, Output> {
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 > 1 {
      0
    } else if ch_id.0 == 0 {
      match &self.input_a_rx {
        &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) => {
          receiver.seqno()
        },
        _ => 0
      }
    } else {
      match &self.input_b_rx {
        &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) => {
          receiver.seqno()
        },
        _ => 0
      }
    }
  }
}

impl<InputA: Send, InputB: Send, Output: Send> OutputCounter for YMergeWrap<InputA, InputB, Output> {
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<InputA: Send, InputB: Send, Output: Send> ConnectableY for YMergeWrap<InputA, InputB, Output> {
  type InputA = InputA;
  type InputB = InputB;

  fn input_a(&mut self) -> &mut ChannelWrapper<InputA> {
    &mut self.input_a_rx
  }

  fn input_b(&mut self) -> &mut ChannelWrapper<InputB> {
    &mut self.input_b_rx
  }
}

impl<InputA: Send, InputB: Send, Output: Send> Task for YMergeWrap<InputA, InputB, Output> {
  fn execute(&mut self) -> Result<(), &'static str> {
    self.state.process(&mut self.input_a_rx,
                       &mut self.input_b_rx,
                       &mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 2 }
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

pub fn new<InputA: Send, InputB: Send, Output: Send>(
    name             : &str,
    output_q_size    : usize,
    ymerge           : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>+Send>)
      -> (Box<YMergeWrap<InputA,InputB,Output>>, Box<ChannelWrapper<Output>>)
{
  let (output_tx,  output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(
      YMergeWrap{
        name          : name.clone(),
        state         : ymerge,
        input_a_rx    : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        input_b_rx    : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(1),
          ReceiverName (name.clone())
        ),
        output_tx     : output_tx,
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
