使用rust开发lox语言解释器

词法分析，给出了TOKEN的类型和相应的数据结构 token.rs；
通过scanner扫描得到TOKEN流，传入parser构建语法分析树AST；
语义分析初步完成，但是没有经过DEBUG，仅确保编译通过

词法分析部分有很多问题，生成AST并不健壮。

file test: cargo run -- test.lox

完成了错误信息格式化输出的处理
完成了类方法调用的BUG修复
部分测试样例已通过

TODO：
部分错误信息还未根据PPT进行调整
