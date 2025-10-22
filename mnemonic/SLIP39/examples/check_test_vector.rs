use slip39::Share;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = "shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed";

    let share = Share::from_mnemonic_string(mnemonic)?;

    println!("Test Vector 4 - First Share:");
    println!("  identifier: {}", share.identifier);
    println!("  group_index: {}", share.group_index);
    println!("  group_threshold: {}", share.group_threshold);
    println!("  group_count: {}", share.group_count);
    println!("  member_index: {}", share.member_index);
    println!("  member_threshold: {}", share.member_threshold);
    println!("  value_len: {}", share.value.len());

    Ok(())
}
