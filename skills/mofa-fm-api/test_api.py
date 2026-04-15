#!/usr/bin/env python3
"""
MoFA FM API 完整测试套件
========================
覆盖所有 API 端点的详细测试
"""

import sys
import json
import time
import traceback
from datetime import datetime

sys.path.insert(0, './tools')
from fm_client import MoFAFMClient, requests


class TestResult:
    """测试结果记录"""
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.skipped = 0
        self.results = []

    def add(self, name, status, data=None, error=None):
        result = {
            "name": name,
            "status": status,
            "timestamp": datetime.now().isoformat(),
            "data": data,
            "error": error
        }
        self.results.append(result)
        if status == "PASS":
            self.passed += 1
        elif status == "FAIL":
            self.failed += 1
        else:
            self.skipped += 1
        return status == "PASS"

    def summary(self):
        total = self.passed + self.failed + self.skipped
        return f"""
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 测试总结
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
总测试数: {total}
✅ 通过: {self.passed}
❌ 失败: {self.failed}
⏭️  跳过: {self.skipped}
通过率: {(self.passed/total*100) if total > 0 else 0:.1f}%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"""


def test_public_apis():
    """测试公开 API（无需认证）"""
    print("\n" + "="*60)
    print("🔓 公开 API 测试 (无需认证)")
    print("="*60)

    results = TestResult()
    client = MoFAFMClient()

    # 1. 健康检查
    try:
        health = client._get("/health/")
        assert "status" in health, "响应缺少 status 字段"
        assert health["status"] == "healthy", f"服务状态异常: {health['status']}"
        print(f"  ✅ 健康检查 - 服务运行正常")
        print(f"     数据库: {health.get('services', {}).get('database', {}).get('status', 'N/A')}")
        print(f"     Redis: {health.get('services', {}).get('redis', {}).get('status', 'N/A')}")
        results.add("health_check", "PASS", health)
    except Exception as e:
        print(f"  ❌ 健康检查失败: {e}")
        results.add("health_check", "FAIL", error=str(e))

    # 2. 获取节目列表
    try:
        shows = client.list_shows()
        assert "results" in shows, "响应缺少 results 字段"
        print(f"  ✅ 节目列表 - 共 {shows.get('count', 0)} 个节目")
        if shows.get('results'):
            sample = shows['results'][0]
            print(f"     示例: {sample.get('title')} (by {sample.get('creator', {}).get('username', 'N/A')})")
        results.add("list_shows", "PASS", shows)
    except Exception as e:
        print(f"  ❌ 节目列表失败: {e}")
        results.add("list_shows", "FAIL", error=str(e))

    # 3. 获取单集列表
    try:
        episodes = client.list_episodes()
        assert "results" in episodes, "响应缺少 results 字段"
        print(f"  ✅ 单集列表 - 共 {episodes.get('count', 0)} 个单集")
        results.add("list_episodes", "PASS", episodes)
    except Exception as e:
        print(f"  ❌ 单集列表失败: {e}")
        results.add("list_episodes", "FAIL", error=str(e))

    # 4. 热搜数据源列表
    try:
        sources = client.list_trending_sources()
        assert len(sources) > 0, "数据源列表为空"
        print(f"  ✅ 热搜数据源 - 共 {len(sources)} 个源")
        results.add("list_trending_sources", "PASS", {"sources": sources, "count": len(sources)})
    except Exception as e:
        print(f"  ❌ 热搜数据源失败: {e}")
        results.add("list_trending_sources", "FAIL", error=str(e))

    # 5. 获取具体热搜数据 (B站)
    try:
        trending = client.get_trending("bilibili")
        assert "data" in trending, "响应缺少 data 字段"
        print(f"  ✅ B站热搜 - 共 {len(trending.get('data', []))} 条")
        if trending.get('data'):
            sample = trending['data'][0]
            print(f"     热榜第一: {sample.get('title', 'N/A')[:40]}...")
        results.add("get_trending_bilibili", "PASS", trending)
    except Exception as e:
        print(f"  ❌ B站热搜失败: {e}")
        results.add("get_trending_bilibili", "FAIL", error=str(e))

    # 6. 搜索功能
    try:
        search_results = client.search("AI")
        print(f"  ✅ 搜索功能 - 找到 {search_results.get('shows', 0)} 个节目")
        results.add("search", "PASS", search_results)
    except Exception as e:
        print(f"  ❌ 搜索功能失败: {e}")
        results.add("search", "FAIL", error=str(e))

    # 7. 搜索建议
    try:
        suggestions = client.search_suggestions("人")
        print(f"  ✅ 搜索建议 - 返回 {len(suggestions)} 条建议")
        results.add("search_suggestions", "PASS", suggestions)
    except Exception as e:
        print(f"  ❌ 搜索建议失败: {e}")
        results.add("search_suggestions", "FAIL", error=str(e))

    # 8. 热门搜索
    try:
        popular = client.popular_searches()
        print(f"  ✅ 热门搜索 - 返回 {len(popular)} 条热门")
        results.add("popular_searches", "PASS", popular)
    except Exception as e:
        print(f"  ❌ 热门搜索失败: {e}")
        results.add("popular_searches", "FAIL", error=str(e))

    # 9. 分类列表
    try:
        categories = client.list_categories()
        print(f"  ✅ 分类列表 - 共 {len(categories)} 个分类")
        results.add("list_categories", "PASS", categories)
    except Exception as e:
        print(f"  ❌ 分类列表失败: {e}")
        results.add("list_categories", "FAIL", error=str(e))

    # 10. 标签列表
    try:
        tags = client.list_tags()
        print(f"  ✅ 标签列表 - 共 {len(tags)} 个标签")
        results.add("list_tags", "PASS", tags)
    except Exception as e:
        print(f"  ❌ 标签列表失败: {e}")
        results.add("list_tags", "FAIL", error=str(e))

    # 11. 平台统计
    try:
        stats = client.get_stats()
        print(f"  ✅ 平台统计 - 获取成功")
        results.add("get_stats", "PASS", stats)
    except Exception as e:
        print(f"  ❌ 平台统计失败: {e}")
        results.add("get_stats", "FAIL", error=str(e))

    # 12. 推荐单集
    try:
        recommended = client.recommended_episodes()
        print(f"  ✅ 推荐单集 - 返回 {len(recommended)} 条推荐")
        results.add("recommended_episodes", "PASS", recommended)
    except Exception as e:
        print(f"  ❌ 推荐单集失败: {e}")
        results.add("recommended_episodes", "FAIL", error=str(e))

    return results


def test_trending_sources_detailed():
    """详细测试所有热搜数据源"""
    print("\n" + "="*60)
    print("🔥 热搜数据源详细测试")
    print("="*60)

    results = TestResult()
    client = MoFAFMClient()

    # 要测试的热门数据源
    test_sources = [
        ("bilibili", "B站"),
        ("zhihu", "知乎"),
        ("weibo", "微博"),
        ("github", "GitHub"),
        ("v2ex", "V2EX"),
        ("douyin", "抖音"),
        ("36kr", "36氪"),
        ("juejin", "掘金"),
        ("sspai", "少数派"),
        ("hackernews", "Hacker News"),
        ("douban-group", "豆瓣小组"),
        ("tieba", "百度贴吧"),
        ("csdn", "CSDN"),
        ("ithome", "IT之家"),
    ]

    for source, name in test_sources:
        try:
            data = client.get_trending(source)
            items = data.get('data', [])
            print(f"  ✅ {name:12s} - {len(items):3d} 条数据")
            results.add(f"trending_{source}", "PASS", {"count": len(items)})
            time.sleep(0.3)  # 避免请求过快
        except Exception as e:
            print(f"  ❌ {name:12s} - 失败: {e}")
            results.add(f"trending_{source}", "FAIL", error=str(e))

    return results


def test_show_details():
    """测试节目详情获取"""
    print("\n" + "="*60)
    print("📺 节目详情测试")
    print("="*60)

    results = TestResult()
    client = MoFAFMClient()

    # 先获取节目列表
    try:
        shows = client.list_shows()
        if not shows.get('results'):
            print("  ⏭️  暂无节目，跳过节目详情测试")
            results.add("show_details", "SKIPPED", reason="no shows available")
            return results

        # 测试第一个节目的详情
        show = shows['results'][0]
        slug = show.get('slug')
        if slug:
            details = client.get_show(slug)
            print(f"  ✅ 节目详情 - {details.get('title', 'N/A')}")
            print(f"     Slug: {slug}")
            print(f"     单集数: {details.get('episodes_count', 0)}")
            print(f"     关注数: {details.get('followers_count', 0)}")
            results.add("get_show_details", "PASS", details)

            # 测试节目下的单集列表 (使用 show_slug 参数)
            try:
                episodes = client._get("/podcasts/episodes/", params={"show_slug": slug})
                print(f"  ✅ 节目单集 - 共 {episodes.get('count', 0)} 个单集")
                results.add("get_show_episodes", "PASS", episodes)
            except Exception as e:
                print(f"  ❌ 节目单集获取失败: {e}")
                results.add("get_show_episodes", "FAIL", error=str(e))
        else:
            results.add("get_show_details", "SKIPPED", reason="no slug available")

    except Exception as e:
        print(f"  ❌ 节目详情测试失败: {e}")
        results.add("show_details", "FAIL", error=str(e))

    return results


def test_episode_details_and_comments():
    """测试单集详情和评论"""
    print("\n" + "="*60)
    print("🎵 单集详情与评论测试")
    print("="*60)

    results = TestResult()
    client = MoFAFMClient()

    try:
        episodes = client.list_episodes()
        if not episodes.get('results'):
            print("  ⏭️  暂无单集，跳过测试")
            results.add("episode_details", "SKIPPED", reason="no episodes available")
            return results

        # 测试第一个单集
        episode = episodes['results'][0]
        episode_id = episode.get('id')

        if episode_id:
            print(f"  ✅ 单集列表 - 找到单集 ID: {episode_id}")

            # 测试播放计数 (这个不需要认证)
            try:
                play_result = client.increment_play_count(episode_id)
                print(f"  ✅ 播放计数 - 成功增加播放数")
                results.add("increment_play_count", "PASS", play_result)
            except Exception as e:
                print(f"  ❌ 播放计数失败: {e}")
                results.add("increment_play_count", "FAIL", error=str(e))

            # 测试评论列表
            try:
                comments = client.list_comments(episode_id)
                print(f"  ✅ 评论列表 - 共 {len(comments)} 条评论")
                results.add("list_comments", "PASS", comments)
            except Exception as e:
                print(f"  ❌ 评论列表失败: {e}")
                results.add("list_comments", "FAIL", error=str(e))
        else:
            results.add("episode_details", "SKIPPED", reason="no episode id available")

    except Exception as e:
        print(f"  ❌ 单集测试失败: {e}")
        results.add("episode_details", "FAIL", error=str(e))

    return results


def test_authenticated_apis(username: str = None, password: str = None, token: str = None):
    """测试需要认证的 API"""
    print("\n" + "="*60)
    print("🔐 认证 API 测试")
    print("="*60)

    results = TestResult()
    client = MoFAFMClient()

    # 如果没有提供 token，尝试登录获取
    if not token and username and password:
        try:
            login_result = client.login(username, password)
            token = client.access_token
            user_info = login_result.get('user', {})
            print(f"  ✅ 登录成功 - 用户: {user_info.get('username', 'N/A')}")
            print(f"     Token 获取成功: {token[:20]}..." if token else "     Token: None")
            results.add("login", "PASS", {"username": user_info.get('username')})
        except Exception as e:
            print(f"  ❌ 登录失败: {e}")
            results.add("login", "FAIL", error=str(e))
            return results
    elif token:
        client.access_token = token
        print(f"  ℹ️  使用提供的 Token: {token[:20]}...")
    else:
        print("  ⏭️  跳过认证测试 (未提供用户名/密码或 token)")
        return results

    # 1. 获取当前用户信息
    try:
        me = client.me()
        print(f"  ✅ 用户信息 - ID: {me.get('id')}, 用户名: {me.get('username')}")
        print(f"     邮箱: {me.get('email')}")
        print(f"     创作者: {me.get('is_creator', False)}")
        results.add("me", "PASS", me)
    except Exception as e:
        print(f"  ❌ 获取用户信息失败: {e}")
        results.add("me", "FAIL", error=str(e))
        return results  # 如果获取用户信息失败，后续测试可能都会失败

    # 2. 成为创作者
    try:
        if not me.get('is_creator'):
            result = client.become_creator()
            print(f"  ✅ 成为创作者 - {result.get('message', '成功')}")
            results.add("become_creator", "PASS", result)
        else:
            print(f"  ⏭️  已是创作者，跳过")
            results.add("become_creator", "SKIPPED", data={"reason": "already creator"})
    except Exception as e:
        print(f"  ❌ 成为创作者失败: {e}")
        results.add("become_creator", "FAIL", error=str(e))

    # 3. 获取我的节目
    try:
        my_shows = client.my_shows()
        print(f"  ✅ 我的节目 - 共 {len(my_shows)} 个节目")
        results.add("my_shows", "PASS", my_shows)
    except Exception as e:
        print(f"  ❌ 获取我的节目失败: {e}")
        results.add("my_shows", "FAIL", error=str(e))

    # 4. 获取生成队列
    try:
        queue = client.generation_queue()
        print(f"  ✅ 生成队列 - 获取成功")
        results.add("generation_queue", "PASS", queue)
    except Exception as e:
        print(f"  ❌ 获取生成队列失败: {e}")
        results.add("generation_queue", "FAIL", error=str(e))

    # 5. 脚本会话列表
    try:
        sessions = client.list_script_sessions()
        print(f"  ✅ 脚本会话 - 共 {len(sessions)} 个会话")
        results.add("list_script_sessions", "PASS", sessions)
    except Exception as e:
        print(f"  ❌ 获取脚本会话失败: {e}")
        results.add("list_script_sessions", "FAIL", error=str(e))

    return results


def save_test_report(all_results, output_file="test_report.json"):
    """保存测试报告"""
    report = {
        "timestamp": datetime.now().isoformat(),
        "summary": {
            "total_passed": sum(r.passed for r in all_results),
            "total_failed": sum(r.failed for r in all_results),
            "total_skipped": sum(r.skipped for r in all_results),
        },
        "details": [
            {
                "name": f"Test-{i+1}",
                "results": r.results
            }
            for i, r in enumerate(all_results)
        ]
    }

    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(report, f, ensure_ascii=False, indent=2)

    print(f"\n📄 测试报告已保存至: {output_file}")
    return report


def main():
    import argparse

    parser = argparse.ArgumentParser(description="MoFA FM API 完整测试")
    parser.add_argument("--username", "-u", help="用户名")
    parser.add_argument("--password", "-p", help="密码")
    parser.add_argument("--token", "-t", help="JWT Token")
    parser.add_argument("--output", "-o", default="test_report.json", help="输出报告文件")
    parser.add_argument("--skip-auth", action="store_true", help="跳过认证测试")
    parser.add_argument("--skip-trending", action="store_true", help="跳过热源详细测试")

    args = parser.parse_args()

    print("""
╔═══════════════════════════════════════════════════════════╗
║         MoFA FM API 完整测试套件                          ║
║         MoFA FM Podcast Platform API Test Suite           ║
╚═══════════════════════════════════════════════════════════╝
    """)

    all_results = []

    # 1. 公开 API 测试
    public_results = test_public_apis()
    all_results.append(public_results)

    # 2. 节目详情测试
    show_results = test_show_details()
    all_results.append(show_results)

    # 3. 单集详情与评论测试
    episode_results = test_episode_details_and_comments()
    all_results.append(episode_results)

    # 4. 热搜数据源详细测试
    if not args.skip_trending:
        trending_results = test_trending_sources_detailed()
        all_results.append(trending_results)

    # 5. 认证 API 测试
    if not args.skip_auth:
        auth_results = test_authenticated_apis(args.username, args.password, args.token)
        all_results.append(auth_results)

    # 汇总结果
    print("\n")
    for result in all_results:
        print(result.summary())

    # 保存报告
    save_test_report(all_results, args.output)

    # 最终状态
    total_failed = sum(r.failed for r in all_results)
    if total_failed > 0:
        print(f"⚠️  有 {total_failed} 个测试失败，请查看详细报告")
        sys.exit(1)
    else:
        print("🎉 所有测试通过！")
        sys.exit(0)


if __name__ == "__main__":
    main()
