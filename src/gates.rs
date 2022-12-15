use bitvec::prelude::*;
use rand::Rng;

pub const ITER: usize = 100;
pub const M: usize = 8;  // number of bits

pub struct Dealer {
    k: Vec<u8>,
    c: u8,
    kc: u8,
}

impl Dealer {
    pub fn new() -> Dealer {
        let mut rng = rand::thread_rng();
        let k = vec![rng.gen::<u8>() % 2, rng.gen::<u8>() % 2];
        let c = rng.gen::<u8>() % 2;
        Dealer { kc: k[c as usize], k: k, c: c, }
    }
}

pub fn secret_share(bit_array: &BitVec<u8>) -> (BitVec<u8>, BitVec<u8>) {
    let mut rng = rand::thread_rng();
    let mut sh_1 = BitVec::<u8>::with_capacity(M);
    let mut sh_2 = BitVec::<u8>::with_capacity(M);
    for i in 0..M {
        sh_1.push(rng.gen::<bool>());
        sh_2.push(sh_1[i] ^ bit_array[i]);
    }
    (sh_1, sh_2)
}

pub fn _reconstruct_shares(ss0: &BitVec<u8>, ss1: &BitVec<u8>) -> BitVec<u8> {
    assert_eq!(ss0.len(), ss1.len());
    let mut reconstructed = BitVec::<u8>::with_capacity(M);
    for (b0, b1) in ss0.iter().zip(ss1.iter()) {
        reconstructed.push(*b0 ^ *b1);
    }
    reconstructed
}

// P0 is the Sender with inputs (m0, m1)
// P1 is the Receiver with inputs (b, mb)
pub fn one_out_of_two_ot(
    dealer: &Dealer,
    receiver_b: u8,
    sender_m: &Vec<u8>) -> u8
{
    let z = receiver_b ^ dealer.c;
    let y = {
        if z == 0 {
            vec![sender_m[0] ^ dealer.k[0], sender_m[1] ^ dealer.k[1]]
        } else {
            vec![sender_m[0] ^ dealer.k[1], sender_m[1] ^ dealer.k[0]]
        }
    };

    y[receiver_b as usize] ^ dealer.kc
}

// OR: z = x | y = ~(~x & ~y)
//   ~(~x & ~y) = ~(~x * ~y) = ~( ~(p0.x + p1.x) * ~(p0.y + p1.y) ) =
//  ~( (~p0.x + p1.x) * (~p0.y + p1.y) ) =
//  ~( (~p0.x * ~p0.y) + (~p0.x * p1.y) + (p1.x * ~p0.y) + (p1.x * p1.y) ) =
//  P0 computes locally ~p0.x * ~p0.y
//  P1 computes locally p1.x * p1.y
//  Both parties compute via OT: ~p0.x * p1.y and p1.x * ~p0.y
pub fn or_gate(x0: bool, y0: bool, x1: bool, y1: bool) -> (bool, bool) {
    let mut rng = rand::thread_rng();

    // Online Phase - P1 receives r0 + p0.x * p1.y
    let r0 = rng.gen::<bool>();
    let dealer = Dealer::new();
    let r0_x0y1 = one_out_of_two_ot(
        &dealer,
        y1 as u8,
        &vec![r0 as u8, (!x0 as u8) ^ (r0 as u8)]
    ) != 0;

    // Online Phase - P0 receives r1 + p1.x * p0.y
    let r1 = rng.gen::<bool>();
    let dealer = Dealer::new();
    let r1_x1y0 = one_out_of_two_ot(
        &dealer,
        !y0 as u8,
        &vec![r1 as u8, (x1 as u8) ^ (r1 as u8)]
    ) != 0;

    // P0
    let share_0 = !( (!x0 & !y0) ^ (r0 ^ r1_x1y0) );

    // P1
    let share_1 = (x1 & y1) ^ (r1 ^ r0_x0y1);

    (share_0, share_1)
}
