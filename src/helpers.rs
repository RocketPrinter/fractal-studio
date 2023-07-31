pub fn extend_array<T: Copy,const N: usize,const M: usize>(arr: &[T;N], default: T) -> [T;M] {
    let mut new_arr = [default; M];
    new_arr[0..N].copy_from_slice(arr);
    new_arr
}