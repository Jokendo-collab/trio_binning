extern crate trio_binning;
use trio_binning::kmer;

fn main() {
    let kmer_str = String::from("ACTGACTGAC");
    println!("Kmer: {}", kmer_str);
    let bits = kmer::kmer_to_bits(&kmer_str).unwrap();
    println!("Bits: {}", bits);
    let kmer_str = kmer::bits_to_kmer(bits, kmer_str.len());
    println!("Kmer: {}", kmer_str.unwrap());
}
