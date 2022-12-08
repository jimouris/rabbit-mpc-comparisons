// https://eprint.iacr.org/2021/119.pdf
// LTBits

mod gates;

use bitvec::prelude::*;
use rand::Rng;

// Returns c = x < R
fn lt_bits(const_r: u8, sh_0: &BitVec<u8>, sh_1: &BitVec<u8>) -> bool {
    let r_bits = const_r.view_bits::<Lsb0>().to_bitvec();
    if gates::DEBUG {
        println!("\tr_bits:   {}", r_bits);
    }

    // Step 1
    let mut y_bits_0 = bitvec![u8, Lsb0; 0; gates::M];
    let mut y_bits_1 = bitvec![u8, Lsb0; 0; gates::M];
    for i in 0..gates::M {
        y_bits_0.set(i, sh_0[i] ^ r_bits[i]);
        y_bits_1.set(i, sh_1[i]);
    }
    if gates::DEBUG {
        println!("\ty_bits_0: {}", y_bits_0);
        println!("\ty_bits_1: {}", y_bits_1);
        println!("\ty_bits  : {}", gates::reconstruct_shares(&y_bits_0, &y_bits_1));
        println!();
    }

    // Step 2
    let mut z_bits_0 = bitvec![u8, Lsb0; 0; gates::M];
    let mut z_bits_1 = bitvec![u8, Lsb0; 0; gates::M];
    for i in 0..gates::M {
        for j in i..gates::M {
            let (or_0, or_1) = gates::or_gate(z_bits_0[i], y_bits_0[j], z_bits_1[i], y_bits_1[j]);

            z_bits_0.set(i, or_0);
            z_bits_1.set(i, or_1);
        }
    }
    z_bits_0.push(false);
    z_bits_1.push(false);
    if gates::DEBUG {
        println!("\tz_bits_0: {}", z_bits_0);
        println!("\tz_bits_1: {}", z_bits_1);
        println!("\tz_bits  : {}", gates::reconstruct_shares(&z_bits_0, &z_bits_1));
        println!();
    }

    // // Step 3
    let mut w_bits_0 = bitvec![u8, Lsb0; 0; gates::M];
    let mut w_bits_1 = bitvec![u8, Lsb0; 0; gates::M];
    for i in 0..gates::M {
        w_bits_0.set(i, z_bits_0[i] ^ z_bits_0[i+1]); // -
        w_bits_1.set(i, z_bits_1[i] ^ z_bits_1[i+1]); // -
    }
    if gates::DEBUG {
        println!("\tr_bits:   {}", r_bits);
        println!("\tw_bits_0: {}", w_bits_0);
        println!("\tw_bits_1: {}", w_bits_1);
        println!("\tw_bits  : {}", gates::reconstruct_shares(&w_bits_0, &w_bits_1));
        println!();
    }

    // Step 4
    let mut sum_0 = 0;
    let mut sum_1 = 0;
    for i in 0..gates::M {
        sum_0 += if r_bits[i] & w_bits_0[i] { 1 } else { 0 };
        sum_1 += if r_bits[i] & w_bits_1[i] { 1 } else { 0 };
    }
    if gates::DEBUG {
        println!("\tsum_0: {}", sum_0);
        println!("\tsum_1: {}", sum_1);
        println!("\tsum  : {}", sum_0 ^ sum_1);
    }

    (sum_0 ^ sum_1) != 0
}

fn main() {
    println!("[LSB, ..., MSB]\n");
    const R: u8 = 128; // public const
    let mut rng = rand::thread_rng();

    for i in 0..gates::ITER {
        let x = rng.gen::<u8>();
        let x_bits = x.view_bits::<Lsb0>().to_bitvec();
        let (x0, x1) = gates::secret_share(&x_bits);

        if gates::DEBUG {
            println!("{}) {} < {}:", i, x, R);
            println!("\tx:        {}", x);
            println!("\tx:        {}", x_bits);
            println!("\tx0:       {}", x0);
            println!("\tx1:       {}", x1);
            println!();
        }
        let lt = lt_bits(R, &x0, &x1);
        println!("{}) {} < {}: {} (expected: {})", i, x, R, lt, x < R);
        assert_eq!(lt, x < R);
        if gates::DEBUG {
            println!("=========================================\n\n");
        }
    }

}
