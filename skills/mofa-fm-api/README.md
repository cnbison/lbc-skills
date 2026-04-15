# MoFA FM API Skill

MoFA FM 播客平台 API 的封装和工具集。

## 快速开始

```bash
# 1. 健康检查
python tools/fm_client.py health

# 2. 获取 B 站热搜
python tools/fm_client.py trending --source bilibili

# 3. 搜索播客
python tools/fm_client.py search --query "AI"
```

## 目录结构

```
mofa-fm-api/
├── SKILL.md                    # Skill 定义文档
├── README.md                   # 本文件
├── tools/
│   └── fm_client.py           # Python API 客户端 + CLI
└── examples/
    ├── basic_usage.py         # 基础使用示例
    └── content_creation_workflow.md  # 内容创作工作流
```

## API 覆盖

- ✅ 认证 (注册/登录/用户信息)
- ✅ 节目 (列表/详情/创建/更新/删除)
- ✅ 单集 (列表/详情/创建/播放)
- ✅ 热搜 (56 个数据源)
- ✅ 搜索 (关键词/建议/热门)
- ✅ 互动 (评论/点赞/关注)
- ✅ 创作者 (我的节目/生成队列)
- ✅ 脚本管理 (脚本会话/AI对话/脚本编辑)

## Python 使用

```python
from fm_client import MoFAFMClient

client = MoFAFMClient()

# 公开 API
shows = client.list_shows()
trending = client.get_trending("bilibili")
results = client.search("AI")

# 需要认证
client.login("username", "password")
me = client.me()
my_shows = client.my_shows()

# 脚本管理 - AI 生成播客脚本
session = client.create_script_session("AI话题讨论", "讨论AI最新进展")
response = client.chat_script_session(session["id"], "帮我写一个关于Claude 3.5的播客脚本")

# 上传参考文件
client.upload_script_reference(session["id"], "/path/to/article.pdf", "参考资料")

# 更新单集脚本（支持Markdown格式，使用【角色名】标签）
script_content = """【主持人】大家好，欢迎收听本期节目
【嘉宾】今天我们来聊聊AI的最新进展..."""
client.update_episode_script(episode_id=123, script=script_content)

# 列出所有脚本会话
sessions = client.list_script_sessions()

# 将脚本会话转为播客单集
episode = client.finalize_script_session(
    session_id=session["id"],
    show_slug="my-show",
    title="AI播客第1期"
)
```

## CLI 使用脚本功能

```bash
# 创建脚本会话
python tools/fm_client.py create-script-session --token $TOKEN \
    --session-title "AI讨论" --session-desc "聊聊AI发展"

# AI 对话生成脚本
python tools/fm_client.py chat-script-session --token $TOKEN \
    --session-id 1 --text "帮我写一个5分钟的播客脚本"

# 上传参考文件
python tools/fm_client.py upload-script-file --token $TOKEN \
    --session-id 1 --file-path ./article.pdf

# 更新单集脚本（从文件读取）
python tools/fm_client.py update-script --token $TOKEN \
    --episode-id 123 --script ./script.md

# 或直接传入脚本内容
python tools/fm_client.py update-script --token $TOKEN \
    --episode-id 123 --script "【主持人】大家好..."
```

## 文档

- API 文档: https://mofa.fm/swagger/
- ReDoc: https://mofa.fm/redoc/
