使用rust开发lox语言解释器

词法分析，给出了TOKEN的类型和相应的数据结构 token.rs；
通过scanner扫描得到TOKEN流，传入parser构建语法分析树AST；
语义分析初步完成，但是没有经过DEBUG，仅确保编译通过


TODO：
词法分析  file test: cargo run -- --input test.lox
语法分析
语义分析