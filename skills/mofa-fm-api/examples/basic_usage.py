#!/usr/bin/env python3
"""
MoFA FM API 基本使用示例
"""

import sys
sys.path.insert(0, '../tools')

from fm_client import MoFAFMClient


def demo_public_apis():
    """演示公开 API（无需认证）"""
    client = MoFAFMClient()

    print("=== 1. 健康检查 ===")
    health = client._get("/health/")
    print(f"服务状态: {health['status']}")
    print(f"数据库: {health['services']['database']['status']}")
    print(f"Redis: {health['services']['redis']['status']}")
    print()

    print("=== 2. 获取节目列表 ===")
    shows = client.list_shows()
    print(f"共 {shows['count']} 个节目")
    for show in shows['results'][:3]:
        print(f"  - {show['title']} (by {show['creator']['username']})")
    print()

    print("=== 3. 获取 B 站热搜 ===")
    trending = client.get_trending("bilibili")
    print(f"来源: {trending['title']} - {trending['type']}")
    print(f"共 {trending['total']} 条，更新时间: {trending['updateTime']}")
    for item in trending['data'][:5]:
        print(f"  - {item['title']} (热度: {item.get('hot', 'N/A')})")
    print()

    print("=== 4. 搜索 ===")
    results = client.search("mofa")
    print(f"找到 {results['shows']} 个节目")
    for show in results.get('shows', [])[:3]:
        print(f"  - {show['title']}")
    print()


def demo_auth_apis(username: str, password: str):
    """演示需要认证的 API"""
    client = MoFAFMClient()

    print("=== 登录 ===")
    result = client.login(username, password)
    print(f"登录成功，用户名: {result.get('user', {}).get('username')}")
    print()

    print("=== 获取当前用户 ===")
    me = client.me()
    print(f"ID: {me['id']}, 用户名: {me['username']}")
    print(f"邮箱: {me['email']}, 创作者: {me['is_creator']}")
    print()

    if not me['is_creator']:
        print("=== 成为创作者 ===")
        result = client.become_creator()
        print(f"结果: {result.get('message')}")
        print()

    print("=== 获取我的节目 ===")
    my_shows = client.my_shows()
    print(f"共 {len(my_shows)} 个节目")
    for show in my_shows:
        print(f"  - {show['title']} ({show['episodes_count']} 单集)")
    print()


def demo_trending_sources():
    """演示所有热搜数据源"""
    client = MoFAFMClient()

    sources = client.list_trending_sources()
    print(f"=== 支持的 {len(sources)} 个热搜数据源 ===")

    # 分类展示
    categories = {
        "社交媒体": ["weibo", "zhihu", "douyin", "kuaishou", "xiaohongshu"],
        "技术": ["github", "juejin", "csdn", "v2ex", "hackernews", "hellogithub"],
        "新闻": ["36kr", "huxiu", "geekpark", "ithome", "sina-news", "netease-news"],
        "视频": ["bilibili", "acfun"],
        "社区": ["douban-group", "tieba", "sspai", "newsmth"],
    }

    for category, items in categories.items():
        found = [s for s in sources if s in items]
        if found:
            print(f"\n{category}: {', '.join(found)}")

    # 获取几个示例
    print("\n=== 获取知乎热搜示例 ===")
    zhihu = client.get_trending("zhihu")
    for item in zhihu['data'][:3]:
        print(f"  - {item['title'][:50]}...")

    print("\n=== 获取 GitHub Trending 示例 ===")
    github = client.get_trending("github")
    for item in github['data'][:3]:
        print(f"  - {item['title']}")


if __name__ == "__main__":
    import sys

    if len(sys.argv) > 1 and sys.argv[1] == "--auth":
        if len(sys.argv) < 4:
            print("Usage: python basic_usage.py --auth <username> <password>")
            sys.exit(1)
        demo_auth_apis(sys.argv[2], sys.argv[3])
    elif len(sys.argv) > 1 and sys.argv[1] == "--trending":
        demo_trending_sources()
    else:
        demo_public_apis()
        print("\n提示: 添加 --auth username password 参数可测试认证接口")
        print("      添加 --trending 参数可查看所有热搜数据源")
