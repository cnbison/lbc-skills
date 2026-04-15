# MoFA FM 内容创作工作流

使用 MoFA FM API 自动化内容创作的工作流示例。

## 工作流 1: 基于热搜自动生成播客单集

```python
#!/usr/bin/env python3
"""
热搜 → AI 生成播客单集

步骤:
1. 获取热搜话题
2. 选择感兴趣的话题
3. AI 生成播客脚本
4. 创建播客单集
"""

from fm_client import MoFAFMClient


def create_episode_from_trending(
    username: str,
    password: str,
    show_slug: str,
    source: str = "zhihu"
):
    """从热搜创建播客单集"""
    client = MoFAFMClient()

    # 1. 登录
    client.login(username, password)
    print(f"✓ 已登录")

    # 2. 获取热搜
    trending = client.get_trending(source)
    print(f"✓ 获取 {trending['title']} {len(trending['data'])} 条热搜")

    # 3. 选择热度最高的话题
    top_item = trending['data'][0]
    title = top_item['title']
    print(f"✓ 选择话题: {title}")

    # 4. AI 生成播客
    result = client._post("/podcasts/episodes/generate-from-source/", {
        "title": title,
        "description": top_item.get('desc', '')[:200],
        "source_url": top_item['url'],
        "show_slug": show_slug,
        "content_type": "podcast",
        "generation_config": {
            "style": " conversational",
            "duration": 10  # 分钟
        }
    }, auth=True)

    print(f"✓ 生成任务已提交: {result.get('task_id')}")
    return result


# 使用示例
# create_episode_from_trending("username", "password", "my-show", "bilibili")
```

## 工作流 2: 每日热搜汇总

```python
def daily_trending_summary():
    """获取多平台热搜并生成汇总"""
    client = MoFAFMClient()

    sources = ["zhihu", "weibo", "bilibili", "github"]
    summary = {}

    for source in sources:
        try:
            data = client.get_trending(source)
            summary[source] = {
                "title": data.get("title", source),
                "top_5": [
                    {"title": item["title"], "hot": item.get("hot", 0)}
                    for item in data.get("data", [])[:5]
                ]
            }
        except Exception as e:
            print(f"获取 {source} 失败: {e}")

    return summary


# 输出格式示例
# {
#   "zhihu": {
#     "title": "知乎",
#     "top_5": [
#       {"title": "今天有什么新闻", "hot": 17140000},
#       ...
#     ]
#   },
#   ...
# }
```

## 工作流 3: 播客数据分析

```python
def analyze_show_performance(show_slug: str):
    """分析节目表现"""
    client = MoFAFMClient()

    show = client.get_show(show_slug)

    # 基础统计
    stats = {
        "节目名称": show['title'],
        "总播放量": show['total_plays'],
        "关注者": show['followers_count'],
        "单集数": show['episodes_count'],
    }

    # 获取单集详情
    episodes_resp = client._get(f"/podcasts/shows/{show_slug}/episodes/")
    episodes = episodes_resp.get('results', [])

    # 分析每集表现
    episode_stats = []
    for ep in episodes:
        episode_stats.append({
            "标题": ep['title'],
            "播放量": ep['play_count'],
            "点赞": ep['like_count'],
            "评论": ep['comment_count'],
            "时长": f"{ep['duration']//60}分" if ep.get('duration') else "未知"
        })

    # 排序
    episode_stats.sort(key=lambda x: x['播放量'], reverse=True)

    return {
        "概览": stats,
        "单集排行": episode_stats[:10]
    }
```

## 工作流 4: 互动自动化

```python
def batch_reply_comments(username: str, password: str, episode_id: int):
    """批量回复评论（示例：感谢所有评论者）"""
    client = MoFAFMClient()
    client.login(username, password)

    # 获取评论
    comments = client.list_comments(episode_id)

    for comment in comments:
        # 只回复没有回复的评论
        if not comment.get('replies'):
            reply_text = f"感谢 @{comment['user']['username']} 的评论！"
            client.create_comment(episode_id, reply_text)
            print(f"回复了 {comment['user']['username']}")


# 注意：实际使用需要控制频率，避免触发限流
```

## 工作流 5: 内容监控

```python
import time


def monitor_new_episodes(show_slug: str, interval: int = 300):
    """监控节目新单集"""
    client = MoFAFMClient()
    last_count = 0

    while True:
        show = client.get_show(show_slug)
        current_count = show['episodes_count']

        if current_count > last_count:
            new_episodes = current_count - last_count
            print(f"🎉 发现 {new_episodes} 个新单集！")
            # 可以在这里添加通知逻辑

        last_count = current_count
        time.sleep(interval)
```

## 工作流 6: 跨平台内容同步

```python
def sync_trending_to_podcast(
    username: str,
    password: str,
    show_slug: str
):
    """将多平台热搜汇总成播客"""
    client = MoFAFMClient()
    client.login(username, password)

    # 收集各平台热搜
    content_parts = []

    for source in ["zhihu", "weibo", "github"]:
        data = client.get_trending(source)
        top_items = data.get('data', [])[:3]

        content_parts.append(f"\n【{data['title']}】")
        for i, item in enumerate(top_items, 1):
            content_parts.append(f"{i}. {item['title']}")

    # 组合内容
    full_content = "\n".join(content_parts)

    # 生成播客
    result = client._post("/podcasts/episodes/generate/", {
        "show_slug": show_slug,
        "title": f"每日热搜汇总 {datetime.now().strftime('%Y-%m-%d')}",
        "content": full_content,
        "generation_config": {
            "style": "news",
            "tone": "professional"
        }
    }, auth=True)

    return result
```

## CLI 命令示例

```bash
# 1. 健康检查
python tools/fm_client.py health

# 2. 获取热搜
python tools/fm_client.py trending --source bilibili
python tools/fm_client.py trending --source zhihu

# 3. 搜索内容
python tools/fm_client.py search --query "AI"

# 4. 登录并获取用户信息（需要账号）
python tools/fm_client.py login --username xxx --password xxx

# 5. 获取节目列表
python tools/fm_client.py shows

# 6. 获取评论
python tools/fm_client.py comments --episode-id 108
```

## 组合使用

```bash
# 获取热搜并格式化输出
python tools/fm_client.py trending --source github | jq '.data[:3] | map({title: .title, hot: .hot})'

# 搜索并提取节目名称
python tools/fm_client.py search --query "技术" | jq '.shows[:3] | map(.title)'

# 获取所有可用的热搜源
python tools/fm_client.py trending-sources | jq '.sources | sort'
```
