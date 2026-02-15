# CodePack

源码打包工具 — 将项目源码按模块打包，方便复制给 AI 或导出为文件。

## 功能

- **拖拽/选择项目文件夹** — 支持拖拽或点击选择
- **自动识别项目类型** — Android / Flutter / Rust / Go / Next.js / Python / Java 等 15+ 种
- **树形文件浏览** — 按目录结构展示，支持勾选/取消
- **代码预览** — 右侧实时预览选中文件内容
- **Token 估算** — 底部显示已选文件数和预估 token 数
- **一键操作** — 复制到剪贴板 / 导出为文件
- **记忆功能** — 自动保存每个项目的勾选状态，下次打开恢复
- **项目元数据** — 自动提取版本号、依赖列表、入口文件等，导出时附加丰富上下文
- **预设系统** — 保存多套勾选方案（如"前端代码"、"后端代码"），一键切换
- **插件系统** — 自定义项目类型识别规则、排除目录、源码扩展名
- **统计面板** — 按语言统计文件数、代码行数、大小占比，可视化分布条

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
| `build.gradle.kts` / `build.gradle` | Android / Gradle |
| `pubspec.yaml` | Flutter / Dart |
| `Cargo.toml` | Rust |
| `go.mod` | Go |
| `pom.xml` | Java / Maven |
| `Package.swift` | Swift |
| `CMakeLists.txt` | C++ / CMake |
| `next.config.*` | Next.js |
| `nuxt.config.*` | Nuxt.js |
| `vite.config.*` | Vite |
| `pyproject.toml` / `requirements.txt` | Python |
| `package.json` | Node.js |
| 其他 | 通用 |

## 插件系统

在系统配置目录下创建 `codepack/plugins/` 文件夹，添加 JSON 插件文件即可扩展项目识别：

```jsonc
// ~/.config/codepack/plugins/unity.json (Linux/macOS)
// %APPDATA%/codepack/plugins/unity.json (Windows)
{
  "name": "Unity",
  "version": "1.0",
  "detect_files": ["ProjectSettings/ProjectVersion.txt"],
  "detect_dirs": ["Assets", "Packages"],
  "exclude_dirs": ["Library", "Temp", "Logs"],
  "source_extensions": ["cs", "shader", "compute"]
}
```

也可以通过应用内的设置面板（齿轮图标）可视化管理插件。

## 预设系统

对同一个项目可以保存多套文件勾选方案，方便在不同导出场景间快速切换。预设持久化保存在配置文件中。

## 配置存储

配置保存在系统 AppData 目录下的 `codepack_config.json`。
