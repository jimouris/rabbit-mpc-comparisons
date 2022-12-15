// https://eprint.iacr.org/2021/119.pdf
// LTBits

mod gates;
mod fastfield;

use bitvec::prelude::*;
use rand::Rng;
use fast_math::log2_raw;
use debug_print::debug_println;

use crate::fastfield::FE;
use crate::fastfield::Share;
use crate::fastfield::Group;

fn get_rand_edabit() -> ((FE, BitVec<u8>), (FE, BitVec<u8>)) {
    let mut rng = rand::thread_rng();
    let r = rng.gen::<u8>() % 64;
    let r_bits = r.view_bits::<Lsb0>().to_bitvec();
    let (r_0_bits, r_1_bits) = gates::secret_share(&r_bits);
    let (r_0, r_1) = FE::new(r as u64).share();
    ((r_0, r_0_bits), (r_1, r_1_bits))
}

// Returns [c] = [R <= x]
fn lt_bits(
    const_r: u8, sh_0: &BitVec<u8>, sh_1: &BitVec<u8>
) -> (u8, u8) {
    let r_bits = const_r.view_bits::<Lsb0>().to_bitvec();

    // Step 1
    let mut y_bits_0 = bitvec![u8, Lsb0; 0; gates::M];
    let mut y_bits_1 = bitvec![u8, Lsb0; 0; gates::M];
    for i in 0..gates::M {
        y_bits_0.set(i, sh_0[i] ^ r_bits[i]);
        y_bits_1.set(i, sh_1[i]);
    }

    // Step 2 - PreOpL
    let log_m = log2_raw(gates::M as f32).ceil() as usize;
    for i in 0..log_m {
        for j in 0..(gates::M / (1 << (i + 1))) {
            let y = ((1 << i) + j * (1 << (i + 1))) - 1;
            for z in 1..(1 << (i + 1)) {
                if y + z < gates::M {
                    let idx_y = gates::M - 1 - y;
                    let (or_0, or_1) = gates::or_gate(
                        y_bits_0[idx_y], y_bits_0[idx_y - z],
                        y_bits_1[idx_y], y_bits_1[idx_y - z]
                    );
                    y_bits_0.set(idx_y - z, or_0);
                    y_bits_1.set(idx_y - z, or_1);
                }
            }
        }
    }
    y_bits_0.push(false);
    y_bits_1.push(false);
    let z_bits_0 = y_bits_0;
    let z_bits_1 = y_bits_1;

    // Step 3
    let mut w_bits_0 = bitvec![u8, Lsb0; 0; gates::M];
    let mut w_bits_1 = bitvec![u8, Lsb0; 0; gates::M];
    for i in 0..gates::M {
        w_bits_0.set(i, z_bits_0[i] ^ z_bits_0[i+1]); // -
        w_bits_1.set(i, z_bits_1[i] ^ z_bits_1[i+1]); // -
    }

    // Step 4
    let mut sum_0 = 0u8;
    let mut sum_1 = 0u8;
    for i in 0..gates::M {
        sum_0 += if r_bits[i] & w_bits_0[i] { 1 } else { 0 };
        sum_1 += if r_bits[i] & w_bits_1[i] { 1 } else { 0 };
    }

    (1 - sum_0.view_bits::<Lsb0>().to_bitvec()[0] as u8, 
    sum_1.view_bits::<Lsb0>().to_bitvec()[0] as u8)
}

// Returns c = x <= R
fn lt_const(const_r: u8, x_0: FE, x_1: FE) -> (u8, u8) {
    let ((r_0, r_0_bits), (r_1, r_1_bits)) = get_rand_edabit();
    let const_m = 1 << 8;

    debug_println!("Params:");
    debug_println!("\tR: {}", const_r);
    debug_println!("\tM: {}", const_m);
    let mut r: FE = Group::zero();
    r.add(&r_0);
    r.add(&r_1);
    debug_println!("\trandom r for edabit: {}", r.value());

    // Step 1
    let mut a_0: FE = Group::zero();
    a_0.add(&x_0);
    a_0.add(&r_0);

    let mut a_1: FE = Group::zero();
    a_1.add(&x_1);
    a_1.add(&r_1);
    
    let b_0 = a_0.clone();
    let mut b_1 = a_1.clone();
    let const_r_fe = FE::new(const_m - const_r as u64);
    b_1.add(&const_r_fe);

    // Step 2
    let mut a: FE = Group::zero();
    a.add(&a_0);
    a.add(&a_1);

    let mut b: FE = Group::zero();
    b.add(&b_0);
    b.add(&b_1);

    debug_println!("Steps 1 and 2 (compute a and b and open them):");
    debug_println!("\ta (= x + r): {}", a.value() as u8);
    debug_println!("\tb (= x + r + M - R): {}", b.value() as u8);

    // Step 3
    let (w1_0, w1_1) = lt_bits(a.value() as u8, &r_0_bits, &r_1_bits);
    let (w2_0, w2_1) = lt_bits(b.value() as u8, &r_0_bits, &r_1_bits);
    let w3 = ((b.value() as u8) < (const_m - const_r as u64) as u8) as u8;
    
    debug_println!("Step 3:");
    debug_println!("\tw1 (LTbits(a <= r) -- LTbits({} <= {})): {}", a.value() as u8, r.value(), w1_0 ^ w1_1);
    debug_println!("\tw2 (LTbits(b <= r) -- LTbits({} <= {})): {}", b.value() as u8, r.value(), w2_0 ^ w2_1);
    debug_println!("\tw3 ((b < M - R) -- {} < {}): {}", b.value() as u8, (const_m - const_r as u64) as u8, w3);
    
    // Step 4
    let w_0 = 1 - (w1_0 ^ w2_0 ^ w3);
    let w_1 = w1_1 ^ w2_1;
    
    debug_println!("Step 4:");   
    debug_println!("\tw (1 - (w1 - w2 + w3)): {}", w_0 ^ w_1);
    debug_println!("\tx < {} : {}", const_r, w_0 ^ w_1 != 0);

    (w_0, w_1)
}

fn main() {
    debug_println!("[LSB, ..., MSB]\n");
    const R: u8 = 1; // public const
    let mut rng = rand::thread_rng();

    for i in 0..gates::ITER {
        let x = rng.gen_range(1..255);
        let x_bits = x.view_bits::<Lsb0>().to_bitvec();
        let (x0, x1) = gates::secret_share(&x_bits);

        // LT Bits: [R <= x]
        let (sum_0, sum_1) = lt_bits(R, &x0, &x1);
        let lt = sum_0 ^ sum_1;
        assert_eq!(lt != 0, R <= x, "LT Bits: {} <= {}", R, x);
        println!("LT Bits {}) {} <= {}: {} (expected: {})", i, R, x, lt, R <= x);

        let (x_0, x_1) = FE::new(x as u64).share();
        println!("input x = {}", x);
        
        // LT Const: [x < R]
        let (w_0, w_1) = lt_const(R, x_0, x_1);
        let lt = w_0 ^ w_1;
        assert_eq!(lt != 0, x <= R, "LT Const: {} <= {}", x, R);
        println!("LT Const {}) {} <= {}: {} (expected: {})", i, x, R, lt, x <= R);
        println!();
    }

}
