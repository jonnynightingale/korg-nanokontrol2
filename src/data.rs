use super::TransportButton;

#[derive(Debug, Default)]
pub struct GroupData {
    pub slider_value: u8,
    pub knob_value: u8,
    pub solo: u8,
    pub mute: u8,
    pub record: u8,
}

#[derive(Debug, Default)]
pub struct Data {
    pub track_rewind: u8,
    pub track_fastforward: u8,
    pub cycle: u8,
    pub set: u8,
    pub marker_rewind: u8,
    pub marker_fastforward: u8,
    pub rewind: u8,
    pub fastforward: u8,
    pub stop: u8,
    pub play: u8,
    pub record: u8,
    pub groups: [GroupData; 8],
}

impl Data {
    pub fn get_transport_button_value(&self, button_type: TransportButton) -> u8 {
        match button_type {
            TransportButton::TrackRewind       => self.track_rewind,
            TransportButton::TrackFastforward  => self.track_fastforward,
            TransportButton::Cycle             => self.cycle,
            TransportButton::Set               => self.set,
            TransportButton::MarkerRewind      => self.marker_rewind,
            TransportButton::MarkerFastforward => self.marker_fastforward,
            TransportButton::Rewind            => self.rewind,
            TransportButton::Fastforward       => self.fastforward,
            TransportButton::Stop              => self.stop,
            TransportButton::Play              => self.play,
            TransportButton::Record            => self.record,
        }
    }
}
