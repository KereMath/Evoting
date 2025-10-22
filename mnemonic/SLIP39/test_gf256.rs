use slip39::shamir::GF256;

fn main() {
    // Test basic GF256 operations
    println!("Testing GF256 arithmetic:");
    
    let a = GF256::new(3);
    let b = GF256::new(5);
    let c = a.mul(b);
    println!("  3 * 5 = {} (expected: 15)", c.value());
    
    let d = GF256::new(200);
    let e = GF256::new(100);
    let f = d.mul(e);
    println!("  200 * 100 = {} (expected: varies)", f.value());
    
    // Test inverse
    let g = GF256::new(7);
    if let Some(g_inv) = g.inverse() {
        let one = g.mul(g_inv);
        println!("  7 * 7^-1 = {} (expected: 1)", one.value());
    }
}
