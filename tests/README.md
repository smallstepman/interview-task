## Structure

This markdown document is used to build test cases for the
`interviewpuzzle` binary (name of the project). Each code
block marked with ` ```colsole ` will be automatically
converted into test case by [trycmd](https://crates.io/crates/trycmd).
Test cases are written using following template:
````
$ <binary-name> <input-parameter>
? <status>
<output>
<mandatory-newline>
````
- `<binary-name>` in our case its `interviewpuzzle`
- This binary will get run using parameter in `$ interviewpuzzle <input-parameter>` (which will get propagated to `clap` for parsing).
- The `<output>` of running above command, will get compared against the <output>.
  - `stderr` gets printed first, followed by `stdout`
- (optional) The line `? <status>` will check against exit code of the processed binary.
- `<mandatory-newline>`, hopefully self-explanatory

Checkout [trycmd](https://docs.rs/trycmd/latest/trycmd/#trycmd) docs to learn more.
Unfortunatelly, naming each test case has not yet been implemented by upstream package ([issue #25](https://github.com/assert-rs/trycmd/issues/25)).
In reallity, I'd just split the test into separate files, and name the files accordingly,
but I'm keeping them bundled for the reader's convenience.

Below you can find all test cases.

## Test cases

### No input

Checking if it fails gracefully when no input is provided.

```console
$ interviewpuzzle 
? failed
interviewpuzzle 

USAGE:
    interviewpuzzle [csv-path]

ARGS:
    <csv-path>    Path to csv file.

OPTIONS:
    -h, --help    Print help information

```

### Basic example (`#01`)
Using an example from PDF file with task description.
This for tests:
- creating new client
- successful deposits
- successful withdrawals
- unsuccessful withdrawals (not enough funds)
```console
$ interviewpuzzle tests/test_cases/01-basic-example.csv
ERROR: Not enough funds in client's account. (tx = type:Withdrawal,client:2,tx:5,amount:Some(3.0),state:None)
client,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false

```

### Don't create new client (`#02`)
See if executing bunch of `Disputes`, `Resolves` and `Chargebacks` agains uninitialized client account
will create new client account (it shouldn't). 
```console
$ interviewpuzzle tests/test_cases/02-dont-create-new-client.csv
ERROR: Attempted to postprocess a non existent transaction. (tx = type:Dispute,client:1,tx:1,amount:None,state:None)
ERROR: Attempted to postprocess a non existent transaction. (tx = type:Resolve,client:1,tx:1,amount:None,state:None)
ERROR: Attempted to postprocess a non existent transaction. (tx = type:Chargeback,client:1,tx:1,amount:None,state:None)

```

### Handling of malformed or non-compatible input (`#03`)
```console
$ interviewpuzzle tests/test_cases/03-malformed-header.csv
? failed
Error: Error(Deserialize { pos: Some(Position { byte: 24, line: 2, record: 1 }), err: DeserializeError { field: None, kind: Message("missing field `client_id`") } })

```

```console
$ interviewpuzzle tests/test_cases/03-malformed-content.csv
? failed
Error: Error(Deserialize { pos: Some(Position { byte: 24, line: 2, record: 1 }), err: DeserializeError { field: None, kind: Message("unknown variant `1`, expected one of `chargeback`, `resolve`, `dispute`, `withdrawal`, `deposit`") } })

```

```console
$ interviewpuzzle tests/test_cases/03-are-you-lost.json
? failed
Error: Error(Deserialize { pos: Some(Position { byte: 2, line: 2, record: 1 }), err: DeserializeError { field: None, kind: Message("missing field `tx_type`") } })

```

### Display decimals with correct precision (`#04`)
```console 
$ interviewpuzzle tests/test_cases/04-decimal-percision.csv
client,available,held,total,locked
1,3.1301,2.1030,5.2331,false
2,4.0400,0.0,4.0400,false

```
