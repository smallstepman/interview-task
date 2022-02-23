## Structure

This markdown document is used to build test cases for the
`interviewpuzzle` binary (name of the project). Each code
block marked with ` ```colsole ` will be automatically
converted into test case by [trycmd](https://crates.io/crates/trycmd).
Test cases are written using following template:

```console ignore
$ <binary-name> <input-parameter>
? <status>
<output>
<mandatory-newline>
```
- `<binary-name>` here its `interviewpuzzle`
- The binary will get run with parameter `$ interviewpuzzle <input-parameter>` (which will later get propagated to `clap` for parsing).
- The `<output>` of running above command, will get compared against the <output>.
  - `stderr` gets printed first, followed by `stdout`
- (optional) The line `? <status>` will check against exit code of the processed binary.
- `<mandatory-newline>`, hopefully self-explanatory

Checkout [trycmd](https://docs.rs/trycmd/latest/trycmd/#trycmd) docs to learn more.
Unfortunatelly, naming each test case has not yet been implemented by upstream package ([issue #25](https://github.com/assert-rs/trycmd/issues/25)), therefore the output of `cargo test` will look like this:

```ignore
running 1 test
Testing tests/README.md:35 ... ok
Testing tests/README.md:58 ... ok
Testing tests/README.md:70 ... ok
Testing tests/README.md:79 ... ok
...
```

In reallity, I'd walk around this issue by splitting this test file into many separate files, and name the files accordingly, but I'm keeping them bundled for the reader's convenience.

Below you can find all test cases.

## Test cases

### No input
This tests for:
- failing gracefully when no input is provided

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
This tests for:
- creating new client
- successful deposits
- successful withdrawals
- unsuccessful withdrawals (not enough funds)
```console
$ interviewpuzzle tests/test_cases/01-basic-example.csv
ERROR: Not enough funds in client's account. (tx = type:Withdrawal,client:2,tx:5,amount:Some(3.0),state:Default)
client,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false

```

### Don't create new client (`#02`)
This tests for:
- executing transaction type: `dispute|resolve|chargeback` against uninitialized client account
should not create new client account 
```console
$ interviewpuzzle tests/test_cases/02-dont-create-new-client.csv
ERROR: Attempted to postprocess a non existent transaction. (tx = type:Dispute,client:1,tx:1,amount:None,state:Default)
ERROR: Attempted to postprocess a non existent transaction. (tx = type:Resolve,client:1,tx:1,amount:None,state:Default)
ERROR: Attempted to postprocess a non existent transaction. (tx = type:Chargeback,client:1,tx:1,amount:None,state:Default)

```

### Handling of malformed or non-compatible input (`#03`)
This tests for:
- malfored header in input file
- malfored content in input file
- non-csv input file
- non-existent file
```console
$ interviewpuzzle tests/test_cases/03-malformed-header.csv
? failed
Error: Error(Deserialize { pos: Some(Position { byte: 24, line: 2, record: 1 }), err: DeserializeError { field: None, kind: Message("unknown field `cliet`, expected one of `tx_type`, `client_id`, `id`, `amount`, `state`") } })

```

```console
$ interviewpuzzle tests/test_cases/03-malformed-content.csv
? failed
Error: Error(Deserialize { pos: Some(Position { byte: 24, line: 2, record: 1 }), err: DeserializeError { field: None, kind: Message("unknown variant `1`, expected one of `chargeback`, `resolve`, `dispute`, `withdrawal`, `deposit`") } })

```

```console
$ interviewpuzzle tests/test_cases/03-are-you-lost.json
? failed
Error: Error(Deserialize { pos: Some(Position { byte: 2, line: 2, record: 1 }), err: DeserializeError { field: None, kind: Message("unknown field `{`, expected one of `tx_type`, `client_id`, `id`, `amount`, `state`") } })

```

```console
$ interviewpuzzle tests/test_cases/03-you-cant-find-me.csv
? failed
Error: Error(Io(Os { code: 2, kind: NotFound, message: "No such file or directory" }))

```

### Display decimals with correct precision (`#04`)
This tests for:
- correctness of floating-point operations
- output decimals formatted with 4-digit precision
```console 
$ interviewpuzzle tests/test_cases/04-decimal-percision.csv
client,available,held,total,locked
1,3.1301,2.1030,5.2331,false
2,4.0400,0.0,4.0400,false

```

### Invalid chargeback request (`#05`)
This tests for:
- *chargeback* happens only when transaction is *disputed*

```console 
$ interviewpuzzle tests/test_cases/05-invalid-chargeback.csv
ERROR: Transaction has been chargedback - no further action is possible. (tx = type:Resolve,client:1,tx:1,amount:None,state:Default)
ERROR: Transaction has been chargedback - no further action is possible. (tx = type:Chargeback,client:1,tx:1,amount:None,state:Default)
ERROR: Transaction has been chargedback - no further action is possible. (tx = type:Chargeback,client:2,tx:2,amount:None,state:Default)
client,available,held,total,locked
1,0.0,4.0,4.0,false
2,4.0,0.0,4.0,false

```

### Non-unique transaction IDs (`#06`)
This tests for:
- refusing to accept `withdrawal|deposit` transactions with non-unique IDs

```console 
$ interviewpuzzle tests/test_cases/06-duplicate-transaction-id.csv
ERROR: Deposit or Withdrawal transaction with the same ID already exists in the ledger. (tx = type:Deposit,client:1,tx:1,amount:Some(4.0),state:Default)
ERROR: Deposit or Withdrawal transaction with the same ID already exists in the ledger. (tx = type:Withdrawal,client:1,tx:1,amount:Some(4.0),state:Default)
client,available,held,total,locked
1,4.0,0.0,4.0,false

```

### Happy path (`#07`)
```console 
$ interviewpuzzle tests/test_cases/07-happy-path.csv
client,available,held,total,locked
1,4.01,4.0,8.01,false

```
