# Encodings

Data encoding is the process of converting structured data into a format that can be
stored and transmitted efficiently. Encoding helps to minimize the size of the data,
improve the speed of transmission, and ensure the integrity of the data being
transmitted. In this research, we will compare various encoding techniques and their
metrics to help developers make informed decisions about the best encoding method
for their use case.

## Run Tests

To run the tests for this research, run the following command:

``` {.sourceCode .bash}

    cargo test -p common-helpers -- --nocapture

```

## Encoding Techniques

There are many encoding techniques available, some of which include:

``` {.sourceCode .text}
● Protocol Buffers
● Avro Binary
● Thrift
● MessagePack
```

Each encoding technique has its own pros and cons, and the choice of encoding
technique will depend on the requirements of the use case.

## Metrics

When evaluating encoding techniques, the following metrics should be considered:

``` {.sourceCode .text}
● Encoded size
● Decoding time
● Compression ratio
● Encoding time
```

## Encoded size

The encoded size is the size of the encoded data after compression. A smaller encoded
size means that less space is required to store the data, which can result in faster
transmission times and lower storage costs.

## Decoding time

The decoding time is the time it takes to convert the encoded data back into its original
form. This metric is important because it impacts the speed at which the data can be
used once it has been received.

## Compression ratio

The compression ratio is a measure of the efficiency of the encoding technique. It is
calculated as the ratio of the size of the original data to the size of the encoded data. A
higher compression ratio means that the data has been compressed more effectively,
which results in smaller encoded sizes and faster transmission times.

## Encoding time

The encoding time is the time it takes to convert the original data into its encoded form.
This metric is important because it impacts the speed at which the data can be
transmitted.

## Encoding Comparison

The following table shows the results of our encoding comparison:

Note: Random data sizes used: 5 kb, 10 kb, 20 kb, 40 kb, 64 kb.

Data Structure:

``` rust
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct TestMessage {
        data: Vec<u8>,
    }
```

Note: It would be much better to pin down a proper data structure for the tests that would implement some derived traits required by the serialization libraries(avro, serde, protobuf, etc). This would help us to get more accurate results and really test the performance of the serialization libraries.

### Table: Thrift

| Encoding Time(ms) | Decoding Time(ms) | Compression Ratio |
|---------------|---------------|-------------------|
| 0.013319       | 0.050818       | 1.005           |
| 0.020193       | 0.085289       | 1.0025          |
| 0.024242       | 0.102876       | 1.00125         |
| 0.026642       | 0.120463       | 1.00065         |
| 0.02873        | 0.188488       | 1.0004          |

### Table: Protocol Buffers

| Encoding Time(ms) | Decoding Time(ms) | Compression Ratio |
|---------------|---------------|-------------------|
| 0.013165       | 0.038698       | 1               |
| 0.025822       | 0.039751       | 1               |
| 0.025822       | 0.102876       | 1               |
| 0.103987       | 0.130529       | 1               |
| 0.145138       | 0.159849       | 1               |

### Table: Avro Binary

| Encoding Time(ms) | Decoding Time(ms) | Compression Ratio |
|---------------|---------------|-------------------|
| 0.155745       | 0.076576       | 1               |
| 0.099921       | 0.057653       | 1               |
| 0.11366        | 0.093713       | 1               |
| 0.231289       | 0.130208       | 1               |
| 0.224128       | 0.193390       | 1               |

### Table: MessagePack

Seems to be a bug in the library, so the results are not accurate.

## Conclusion

The results show the performance of three serialization formats: Thrift, Protocol Buffers and Avro Binary. The data size used in the tests vary from 5,000 to 64,000 bytes.

In terms of encoding time, Protocol Buffers is the fastest, followed by Thrift, while Avro Binary has the slowest encoding time. The decoding time is faster in Thrift compared to the other two formats. The compression ratios of Thrift, Protocol Buffers and Avro Binary are 1.005, 1 and 1 respectively, with the highest compression ratio in Thrift.

Overall, the results suggest that Protocol Buffers is the fastest in terms of encoding time, Thrift is faster in decoding time and has a higher compression ratio, while Avro Binary has the slowest encoding time but reasonable decoding time. The choice of the serialization format would depend on the specific requirements and trade-offs between encoding time, decoding time, and compression ratio.
