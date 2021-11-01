pub fn read(memspace: &[u8; 0xffff], addr: u16) -> u8
{
    match addr 
    {
        _ => return memspace[addr as usize]
    }
    
}

pub fn write(memspace: &mut[u8; 0xffff], addr: u16, data: u8)
{
    match addr
    {
        a if a > 0xe000 => { println!("Attempt to write byte {0} to ROM at address {1}!", data, addr) },
        _ => memspace[addr as usize] = data
    }

    return;
}