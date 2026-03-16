使用rust开发lox语言解释器

词法分析，给出了TOKEN的类型和相应的数据结构 token.rs；
通过scanner扫描得到TOKEN流，传入parser构建语法分析树AST；
语义分析初步完成，但是没有经过DEBUG，仅确保编译通过

file test: cargo run -- test.lox
使用自动化脚本测试需要：
cd 到test_runner目录  cargo build --release
然后  cargo run --release
由于是硬编码，所以添加新的测试样例的话需要将main.rs中for的索引范围增加

测试样例已全部通过



