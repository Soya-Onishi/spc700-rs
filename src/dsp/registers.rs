struct DspRegisters {
    adsr_low: [u8; 8],
    adsr_high: [u8; 8],
    gain: [u8; 8],
    env: [u8; 8],
    out: [u8; 8],
}
