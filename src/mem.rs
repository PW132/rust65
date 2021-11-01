pub fn read(memspace: &[u8; 0xffff], addr: u16) -> u8
{
    return memspace[addr as usize];
}

pub fn write(memspace: &mut[u8; 0xffff], addr: u16, data: u8)
{
    if addr >= 0xe000
    {
        println!("Attempted to write byte {0} to ROM at address {1}!", data, addr);
    }
    else 
    {
        memspace[addr as usize] = data;
    }

    return;
}