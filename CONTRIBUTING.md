# 贡献指南

感谢您对 MC Server Panel 项目的兴趣！🎉

本文档将帮助您了解如何为项目做出贡献。

---

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发环境设置](#开发环境设置)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [Pull Request 流程](#pull-request-流程)
- [报告问题](#报告问题)
- [功能请求](#功能请求)

---

## 📜 行为准则

我们致力于为项目创造一个友好、包容的社区。所有参与者必须遵守以下原则：

1. **友善和尊重** - 尊重他人观点和贡献
2. **包容性** - 欢迎不同背景和技能水平的贡献者
3. **专业性** - 保持建设性的讨论
4. **诚实** - 对错误和不确定性保持透明

---

## 🤝 如何贡献

### 🐛 报告 Bug

如果您发现 Bug，请通过以下方式报告：

1. **检查是否已存在** - 搜索现有 issues
2. **创建新 Issue** - 使用 Bug 报告模板
3. **提供详细信息**:
   - 清晰的问题描述
   - 复现步骤
   - 预期 vs 实际行为
   - 环境信息 (OS, Rust 版本等)
   - 相关日志和截图

### 📝 改进文档

文档改进总是受欢迎的！

- 修正拼写和语法错误
- 改进现有说明
- 添加缺失的文档
- 翻译成其他语言

### 💻 代码贡献

#### 流程

1. **Fork 仓库** - 创建您自己的 fork
2. **克隆您的 Fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/minecraft-admin.git
   cd minecraft-admin
   ```
3. **创建功能分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或修复分支
   git checkout -b fix/issue-number-brief-description
   ```
4. **进行更改** - 编写代码
5. **测试更改** - 确保测试通过
6. **提交更改** - 使用规范的提交信息
7. **推送到您的 Fork**
   ```bash
   git push origin feature/your-feature-name
   ```
8. **创建 Pull Request** - 详细描述您的更改

---

## 🛠 开发环境设置

### 前置要求

- **Rust**: 1.70+
- **Node.js**: 18+
- **npm**: 9+

### 克隆和设置

```bash
# 克隆仓库
git clone https://github.com/mc-server-panel/minecraft-admin.git
cd minecraft-admin

# 添加上游仓库
git remote add upstream https://github.com/mc-server-panel/minecraft-admin.git

# 创建开发分支
git checkout -b develop
```

### 后端开发

```bash
cd backend

# 安装 Rust 依赖
cargo fetch

# 运行开发服务器
cargo run

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化
cargo fmt
```

### 前端开发

```bash
cd frontend

# 安装依赖
npm install

# 运行开发服务器
npm run dev

# 运行测试
npm test

# 代码检查
npm run lint

# 格式化
npm run format
```

---

## 📏 代码规范

### Rust (后端)

- **遵循 `cargo fmt` 格式**
- **遵循 `clippy` 检查**
- **添加必要的文档注释**
- **编写单元测试**

```rust
/// 函数功能描述
///
/// # 参数
/// * `input` - 输入参数描述
///
/// # 示例
/// ```
/// let result = my_function(value);
/// ```
pub fn my_function(input: Type) -> ReturnType {
    // 实现
}
```

### TypeScript/JavaScript (前端)

- **遵循 ESLint 规则**
- **使用有意义变量名**
- **添加 JSDoc 注释**
- **组件使用函数式写法**

```typescript
/**
 * 组件功能描述
 * @param props - 属性说明
 * @returns React 元素
 */
export function MyComponent({ prop1, prop2 }: Props) {
  return <div>{/* JSX */}</div>;
}
```

---

## 📝 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

### 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 类型 (Type)

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档更改 |
| `style` | 代码格式 (不影响功能) |
| `refactor` | 代码重构 |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `build` | 构建系统相关 |
| `ci` | CI/CD 相关 |
| `chore` | 其他更改 |

### 范围 (Scope)

可选，用于标识更改的模块：

- `api` - API 相关
- `auth` - 认证相关
- `ui` - 用户界面
- `core` - 核心功能
- `docs` - 文档
- `test` - 测试

### 示例

```
feat(auth): 添加 TOTP 双因素认证

- 添加 TOTP 密钥生成功能
- 实现验证码验证
- 添加 QR 码显示

Closes #123
```

---

## 🔄 Pull Request 流程

### 创建 PR 前

1. ✅ 代码遵循规范
2. ✅ 所有测试通过
3. ✅ 文档已更新
4. ✅ commit 信息规范
5. ✅ 从最新 main 分支 rebase

### PR 模板

```markdown
## 描述
<!-- 清晰描述这个 PR 的目的 -->

## 更改类型
- [ ] 🐛 Bug 修复
- [ ] ✨ 新功能
- [ ] 📖 文档
- [ ] ♻️ 重构
- [ ] ⚡ 性能
- [ ] 🧪 测试

## 测试
<!-- 描述如何测试这些更改 -->

## 截图 (如适用)
<!-- 添加 UI 更改的截图 -->

## 检查清单
- [ ] 我的代码遵循项目的代码规范
- [ ] 我已经进行了自我代码审查
- [ ] 我已经添加了必要的文档注释
- [ ] 我的更改没有产生新的警告
- [ ] 我已经添加了测试或更新了现有测试
- [ ] 所有测试都通过了
```

### Review 流程

1. **自动检查** - CI/CD 运行测试和检查
2. **Maintainer Review** - 项目维护者审核
3. **讨论** - 如有需要，进行讨论
4. **合并** - 审核通过后合并

---

## 🐛 报告问题

### Issue 模板

```markdown
## 问题描述
<!-- 清晰简洁地描述问题 -->

## 复现步骤
1. <!-- 第一步 -->
2. <!-- 第二步 -->
3. <!-- ... -->

## 预期行为
<!-- 描述您期望发生的行为 -->

## 实际行为
<!-- 描述实际发生的行为 -->

## 环境信息
- 操作系统: [e.g. Ubuntu 22.04]
- Rust 版本: [e.g. 1.70.0]
- Node 版本: [e.g. 18.0.0]
- 包版本/Commit: [e.g. v2.0.0]

## 日志
<!-- 添加相关日志 -->

## 截图 (如适用)
<!-- 添加截图 -->
```

---

## ✨ 功能请求

### Feature Request 模板

```markdown
## 功能描述
<!-- 清晰描述您想要的功能 -->

## 使用场景
<!-- 描述这个功能解决什么问题 -->

## 建议的解决方案
<!-- 描述您认为如何实现这个功能 -->

## 替代方案
<!-- 描述您考虑过的其他解决方案 -->

## 其他信息
<!-- 添加任何其他相关信息 -->
```

---

## 🎯 开发优先级

如果您想贡献代码，可以从以下开始：

### 🐛 优先级高 - 适合新手

- 文档改进
- 单元测试覆盖
- 小型 Bug 修复
- 代码格式化

### 📊 优先级中 - 中级难度

- 现有功能改进
- 性能优化
- 错误处理改进
- 测试增强

### 🚀 优先级低 - 高级难度

- 新功能开发
- 架构重构
- 安全增强
- 性能优化

---

## 📞 联系方式

- **GitHub Issues**: [报告问题](https://github.com/mc-server-panel/minecraft-admin/issues)
- **讨论区**: [GitHub Discussions](https://github.com/mc-server-panel/minecraft-admin/discussions)

---

## 📜 许可证

通过贡献代码，您同意将您的作品以 MIT 许可证发布。

---

感谢您的贡献！ 🚀
