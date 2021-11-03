pub struct Segment<'a>
{
    pub data: &'a mut [u8],
    pub start_addr: u16,
    pub write_enabled: bool,
    pub read_enabled: bool
}


pub fn read(memspace: &[Segment], addr: u16) -> u8 //bus arbitration for reading bytes
{
    let mut read_byte: u8 = 0;
    for bank in memspace
    {
        if addr >= bank.start_addr && addr < (bank.data.len() as u16 + bank.start_addr)
        {
            if bank.read_enabled
            {
                read_byte = bank.data[(addr - bank.start_addr) as usize];
                break;
            }
        }   
    }
    return read_byte;
}


pub fn write(memspace: &mut[Segment], addr: u16, data: u8) //bus arbitration for writing bytes
{
    for bank in memspace
    {
        if addr >= bank.start_addr && addr < (bank.data.len() as u16 + bank.start_addr)
        {
            if bank.write_enabled
            {
                bank.data[(addr - bank.start_addr) as usize] = data;
                break;
            }
        }
    }

    return;
}