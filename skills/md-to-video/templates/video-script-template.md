# {TITLE} · 视频分页脚本

> 由 md-to-video skill 自动生成
> 生成时间: {TIMESTAMP}
> 幻灯片数: {SLIDE_COUNT}
> 旁白声音: {VOICE}
> 风格: {STYLE}

---

## Page 1: 封面

**标题**: {TITLE}
**副标题**: {SUBTITLE}

<!--
  Type: cover
  Duration: {COVER_DURATION}s (静态)
  Audio: none
-->

## Page 2: {PAGE2_TITLE}

{PAGE2_NARRATION}

<!--
  Type: content
  Duration: auto (match audio)
  Audio: page02.mp3
-->

## Page 3: {PAGE3_TITLE}

{PAGE3_NARRATION}

<!--
  Type: content
  Duration: auto (match audio)
  Audio: page03.mp3
-->

<!-- Repeat for each content page... -->

## Page N: 结尾

{ENDING_NARRATION}

<!--
  Type: ending
  Duration: {ENDING_DURATION}s (静态)
  Audio: none
-->
