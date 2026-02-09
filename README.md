# CodePack

源码打包工具 — 将项目源码按模块打包，方便复制给 AI 或导出为文件。

## 功能

- **拖拽/选择项目文件夹** — 支持拖拽或点击选择
- **自动识别项目类型** — Android / Flutter / Next.js / Python / 通用
- **树形文件浏览** — 按目录结构展示，支持勾选/取消
- **代码预览** — 右侧实时预览选中文件内容
- **Token 估算** — 底部显示已选文件数和预估 token 数
- **一键操作** — 复制到剪贴板 / 导出为文件
- **记忆功能** — 自动保存每个项目的勾选状态，下次打开恢复

## 技术栈

- **Tauri v2** — 桌面应用框架
- **Vue 3 + TypeScript** — 前端
- **Tailwind CSS** — 样式（深色主题）
- **Rust** — 后端文件扫描与处理

## 自动排除目录

`node_modules` `build` `dist` `.gradle` `.idea` `.vscode` `__pycache__` `.git` `target` `.next` `venv` 等

## 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npx tauri dev

# 构建生产版本
npx tauri build
```

## 项目识别规则

| 文件 | 项目类型 |
|------|---------|
| `build.gradle.kts` / `build.gradle` | Android |
| `pubspec.yaml` | Flutter |
| `next.config.*` | Next.js |
| `pyproject.toml` / `requirements.txt` | Python |
| 其他 | 通用 |

## 配置存储

配置保存在系统 AppData 目录下的 `codepack_config.json`。
