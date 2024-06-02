foreign_typemap!(
    ($p:r_type) &'a [u8] => jbyteArray {
        let slice = unsafe { std::mem::transmute::<&[u8], &[i8]>($p) };
        let raw = JavaByteArray::from_slice_to_raw(slice, env);
        $out = raw;
    };
    ($p:f_type) => "jbyteArray";
    ($p:r_type) &'a [u8] <= jbyteArray {
        let arr = JavaByteArray::new(env, $p);
        let slice = arr.to_slice();
        let slice = unsafe { std::mem::transmute::<&[i8], &[u8]>(slice) };
        $out = slice;
    };
    ($p:f_type) <= "jbyteArray";
);
