use crate::protocol_buf::ProtocolBufEncoding;

#[test]
fn test_protocol_buf_encoding() {
    let data_sizes = [10, 100, 1000, 10000, 100000];
    let encoder = ProtocolBufEncoding;
    let mut results = vec![];

    for size in data_sizes.iter() {
        let data = vec![0u8; *size];
        let start = Instant::now();
        let encoded = encoder.encode(&data);
        let end = start.elapsed();
        let encoding_time = end.as_secs_f64();

        let start = Instant::now();
        let decoded = encoder.decode(&encoded);
        let end = start.elapsed();
        let decoding_time = end.as_secs_f64();

        let metrics = encoder.get_metrics(&data);
        results.push((
            size,
            metrics.encoded_size,
            encoding_time,
            decoding_time,
            metrics.compression_ratio,
        ));
    }

    println!("Data size | Encoded size | Encoding time | Decoding time | Compression ratio");
    for result in results.iter() {
        println!(
            "{} | {} | {} | {} | {}",
            result.0, result.1, result.2, result.3, result.4
        );
    }
}
