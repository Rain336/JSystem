pub enum AlphaComparisonCondition {
    NeverPass = 0x00,
    LessThan = 0x01,
    LessThanOrEqual = 0x02,
    Equal = 0x03,
    NotEqual = 0x04,
    GratherThanOrEqual = 0x05,
    GratherThan = 0x06,
    AlwaysPass = 0x07,
}

pub enum AlphaComparisonOperation {
    AND = 0x00,
    OR = 0x01,
    XOR = 0x02,
    NOR = 0x03,
}

pub struct AlphaComparison {
    condition: u8,
    operation: AlphaComparisonOperation,
    value_0: u8,
    value_1: u8,
}

pub struct ChannelControl {
    color_source: u8,
    alpha_source: u8,
}

pub struct Material {
    name: String,
    fore_color: [u16; 4],
    back_color: [u16; 4],
    color_register_3: [u16; 4],
    tev_color_1: [u8; 4],
    tev_color_2: [u8; 4],
    tev_color_3: [u8; 4],
    tev_color_4: [u8; 4],
    material_color: Option<[u16; 4]>,
    channel_control: Option<ChannelControl>,
    alpha_comparision: Option<AlphaComparison>,
}

pub struct MaterialSection(Vec<Material>);
