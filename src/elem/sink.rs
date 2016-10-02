use super::super::{Task, ChannelWrapper, ChannelId, SenderName,
  ReceiverChannelId, ReceiverName, SenderChannelId, ChannelPosition
};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};
use super::counter::{InputCounter};

pub trait Sink {
  type InputType : Send;

  fn process(
    &mut self,
    input: &mut ChannelWrapper<Self::InputType>) -> Result<(), &'static str>;
}

pub struct SinkWrap<Input: Send> {
  name      : String,
  state     : Box<Sink<InputType=Input>+Send>,
  input_rx  : ChannelWrapper<Input>,
}

impl<Input: Send> IdentifiedInput for SinkWrap<Input> {
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 != 0 {
      None
    } else {
      match &self.input_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None,
      }
    }
  }
}

impl<Input: Send> InputCounter for SinkWrap<Input> {
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 == 0 {
      if let &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) = &self.input_rx {
        receiver.seqno()
      } else {
        0
      }
    } else {
      0
    }
  }
}

impl<Input: Send> Connectable for SinkWrap<Input> {
  type Input = Input;

  fn input(&mut self) -> &mut ChannelWrapper<Input> {
    &mut self.input_rx
  }
}

impl<Input: Send> Task for SinkWrap<Input> {
  fn execute(&mut self) -> Result<(), &'static str> {
    self.state.process(&mut self.input_rx)
  }

  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 0 }

  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    self.get_input_id(ch_id)
  }

  fn input_channel_pos(&self, ch_id: ReceiverChannelId) -> ChannelPosition {
    ChannelPosition( self.get_rx_count(ch_id) )
  }

  fn output_channel_pos(&self, _ch_id: SenderChannelId) -> ChannelPosition { ChannelPosition(0) }
}

pub fn new<Input: Send>(
    name   : &str,
    sink   : Box<Sink<InputType=Input>+Send>)
      -> Box<SinkWrap<Input>>
{
  let name = String::from(name);
  Box::new(
    SinkWrap{
      name          : String::from(name.clone()),
      state         : sink,
      input_rx      : ChannelWrapper::ReceiverNotConnected(
        ReceiverChannelId(0),
        ReceiverName (name)
      ),
    }
  )
}
