use slip39::Share;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First, let's verify LOG/EXP tables match Python
    println!("=== LOG/EXP Table Verification ===");

    // Generate tables same way as our code
    let mut exp_table = [0u8; 256];
    let mut poly: u16 = 1;
    for i in 0..255 {
        exp_table[i] = poly as u8;
        poly = (poly << 1) ^ poly;
        if poly & 0x100 != 0 {
            poly ^= 0x11B;
        }
    }
    exp_table[255] = exp_table[0];

    let mut log_table = [0u8; 256];
    for (i, &exp_val) in exp_table.iter().enumerate().take(255) {
        log_table[exp_val as usize] = i as u8;
    }

    println!("First 10 EXP values: {:?}", &exp_table[0..10]);
    println!("First 10 LOG values: {:?}", &log_table[0..10]);
    println!("LOG[1]={}, EXP[0]={}", log_table[1], exp_table[0]);
    println!("LOG[2]={}, EXP[1]={}", log_table[2], exp_table[1]);
    println!("LOG[3]={}, EXP[25]={}", log_table[3], exp_table[25]);

    // Now test vector 4
    println!("\n=== Test Vector 4: 2-of-3 Reconstruction ===");
    let mnemonics = vec![
        "shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed",
        "shadow pistol academic acid actress prayer class unknown daughter sweater depict flip twice unkind craft early superior advocate guest smoking",
    ];

    let shares: Vec<Share> = mnemonics
        .iter()
        .map(|m| Share::from_mnemonic_string(m).unwrap())
        .collect();

    println!("Share 0: member_index={}, x_coord={}", shares[0].member_index, shares[0].member_index + 1);
    println!("  First 4 bytes: {}", hex::encode(&shares[0].value[..4]));
    println!("Share 1: member_index={}, x_coord={}", shares[1].member_index, shares[1].member_index + 1);
    println!("  First 4 bytes: {}", hex::encode(&shares[1].value[..4]));

    // Manual interpolation at x=255 for first byte
    let x = 255u8;
    let x1 = shares[0].member_index + 1;
    let y1 = shares[0].value[0];
    let x2 = shares[1].member_index + 1;
    let y2 = shares[1].value[0];

    println!("\n=== Manual Interpolation at x={} for byte 0 ===", x);
    println!("Points: ({}, {}), ({}, {})", x1, y1, x2, y2);

    // Compute log_prod
    let diff1 = x1 ^ x;
    let diff2 = x2 ^ x;
    let log_prod = log_table[diff1 as usize] as u32 + log_table[diff2 as usize] as u32;
    println!("diff1 = {} ^ {} = {}, LOG[{}] = {}", x1, x, diff1, diff1, log_table[diff1 as usize]);
    println!("diff2 = {} ^ {} = {}, LOG[{}] = {}", x2, x, diff2, diff2, log_table[diff2 as usize]);
    println!("log_prod = {} + {} = {}", log_table[diff1 as usize], log_table[diff2 as usize], log_prod);

    // Point 1 contribution
    println!("\n--- Point 1 ({}, {}) ---", x1, y1);
    let mut log_basis1 = log_prod as i32;
    println!("Starting log_basis = {}", log_basis1);

    log_basis1 -= log_table[diff1 as usize] as i32;
    println!("After subtracting LOG[{}] = {}: log_basis = {}", diff1, log_table[diff1 as usize], log_basis1);

    let diff_11 = x1 ^ x1;
    let diff_12 = x1 ^ x2;
    let sum_diffs1 = log_table[diff_11 as usize] as i32 + log_table[diff_12 as usize] as i32;
    println!("diff_11 = {} ^ {} = {}, LOG[{}] = {}", x1, x1, diff_11, diff_11, log_table[diff_11 as usize]);
    println!("diff_12 = {} ^ {} = {}, LOG[{}] = {}", x1, x2, diff_12, diff_12, log_table[diff_12 as usize]);
    println!("sum_diffs = {} + {} = {}", log_table[diff_11 as usize], log_table[diff_12 as usize], sum_diffs1);

    log_basis1 -= sum_diffs1;
    println!("After subtracting sum_diffs: log_basis = {}", log_basis1);

    log_basis1 = log_basis1.rem_euclid(255);
    println!("After rem_euclid(255): log_basis = {}", log_basis1);

    let contrib1 = if y1 == 0 {
        0
    } else {
        let log_y1 = log_table[y1 as usize] as i32;
        let log_result = (log_y1 + log_basis1).rem_euclid(255);
        println!("LOG[y1={}] = {}", y1, log_y1);
        println!("log_result = {} + {} = {} mod 255 = {}", log_y1, log_basis1, log_y1 + log_basis1, log_result);
        let contrib = exp_table[log_result as usize];
        println!("EXP[{}] = {}", log_result, contrib);
        contrib
    };
    println!("Contribution from point 1: {}", contrib1);

    // Point 2 contribution
    println!("\n--- Point 2 ({}, {}) ---", x2, y2);
    let mut log_basis2 = log_prod as i32;
    println!("Starting log_basis = {}", log_basis2);

    log_basis2 -= log_table[diff2 as usize] as i32;
    println!("After subtracting LOG[{}] = {}: log_basis = {}", diff2, log_table[diff2 as usize], log_basis2);

    let diff_21 = x2 ^ x1;
    let diff_22 = x2 ^ x2;
    let sum_diffs2 = log_table[diff_21 as usize] as i32 + log_table[diff_22 as usize] as i32;
    println!("diff_21 = {} ^ {} = {}, LOG[{}] = {}", x2, x1, diff_21, diff_21, log_table[diff_21 as usize]);
    println!("diff_22 = {} ^ {} = {}, LOG[{}] = {}", x2, x2, diff_22, diff_22, log_table[diff_22 as usize]);
    println!("sum_diffs = {} + {} = {}", log_table[diff_21 as usize], log_table[diff_22 as usize], sum_diffs2);

    log_basis2 -= sum_diffs2;
    println!("After subtracting sum_diffs: log_basis = {}", log_basis2);

    log_basis2 = log_basis2.rem_euclid(255);
    println!("After rem_euclid(255): log_basis = {}", log_basis2);

    let contrib2 = if y2 == 0 {
        0
    } else {
        let log_y2 = log_table[y2 as usize] as i32;
        let log_result = (log_y2 + log_basis2).rem_euclid(255);
        println!("LOG[y2={}] = {}", y2, log_y2);
        println!("log_result = {} + {} = {} mod 255 = {}", log_y2, log_basis2, log_y2 + log_basis2, log_result);
        let contrib = exp_table[log_result as usize];
        println!("EXP[{}] = {}", log_result, contrib);
        contrib
    };
    println!("Contribution from point 2: {}", contrib2);

    let result = contrib1 ^ contrib2;
    println!("\nFinal result: {} ^ {} = {}", contrib1, contrib2, result);
    println!("Expected first byte: 0xb4 (180)");

    Ok(())
}
