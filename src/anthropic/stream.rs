//! 娴佸紡鍝嶅簲澶勭悊妯″潡
//!
//! 瀹炵幇 Kiro 鈫?Anthropic 娴佸紡鍝嶅簲杞崲鍜?SSE 鐘舵€佺鐞?

use std::collections::HashMap;

use serde_json::json;
use uuid::Uuid;

use crate::kiro::model::events::Event;

/// 鎵惧埌灏忎簬绛変簬鐩爣浣嶇疆鐨勬渶杩戞湁鏁圲TF-8瀛楃杈圭晫
///
/// UTF-8瀛楃鍙兘鍗犵敤1-4涓瓧鑺傦紝鐩存帴鎸夊瓧鑺備綅缃垏鐗囧彲鑳戒細鍒囧湪澶氬瓧鑺傚瓧绗︿腑闂村鑷磒anic銆?
/// 杩欎釜鍑芥暟浠庣洰鏍囦綅缃悜鍓嶆悳绱紝鎵惧埌鏈€杩戠殑鏈夋晥瀛楃杈圭晫銆?
fn find_char_boundary(s: &str, target: usize) -> usize {
    if target >= s.len() {
        return s.len();
    }
    if target == 0 {
        return 0;
    }
    // 浠庣洰鏍囦綅缃悜鍓嶆悳绱㈡湁鏁堢殑瀛楃杈圭晫
    let mut pos = target;
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// 闇€瑕佽烦杩囩殑鍖呰９瀛楃
///
/// 褰?thinking 鏍囩琚繖浜涘瓧绗﹀寘瑁规椂锛岃涓烘槸鍦ㄥ紩鐢ㄦ爣绛捐€岄潪鐪熸鐨勬爣绛撅細
/// - 鍙嶅紩鍙?(`)锛氳鍐呬唬鐮?
/// - 鍙屽紩鍙?(")锛氬瓧绗︿覆
/// - 鍗曞紩鍙?(')锛氬瓧绗︿覆
const QUOTE_CHARS: &[u8] = &[
    b'`', b'"', b'\'', b'\\', b'#', b'!', b'@', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'-',
    b'_', b'=', b'+', b'[', b']', b'{', b'}', b';', b':', b'<', b'>', b',', b'.', b'?', b'/',
];

/// 妫€鏌ユ寚瀹氫綅缃殑瀛楃鏄惁鏄紩鐢ㄥ瓧绗?
fn is_quote_char(buffer: &str, pos: usize) -> bool {
    buffer
        .as_bytes()
        .get(pos)
        .map(|c| QUOTE_CHARS.contains(c))
        .unwrap_or(false)
}

/// 鏌ユ壘鐪熸鐨?thinking 缁撴潫鏍囩锛堜笉琚紩鐢ㄥ瓧绗﹀寘瑁癸紝涓斿悗闈㈡湁鍙屾崲琛岀锛?
///
/// 褰撴ā鍨嬪湪鎬濊€冭繃绋嬩腑鎻愬埌 `</thinking>` 鏃讹紝閫氬父浼氱敤鍙嶅紩鍙枫€佸紩鍙风瓑鍖呰９锛?
/// 鎴栬€呭湪鍚屼竴琛屾湁鍏朵粬鍐呭锛堝"鍏充簬 </thinking> 鏍囩"锛夈€?
/// 杩欎釜鍑芥暟浼氳烦杩囪繖浜涙儏鍐碉紝鍙繑鍥炵湡姝ｇ殑缁撴潫鏍囩浣嶇疆銆?
///
/// 璺宠繃鐨勬儏鍐碉細
/// - 琚紩鐢ㄥ瓧绗﹀寘瑁癸紙鍙嶅紩鍙枫€佸紩鍙风瓑锛?
/// - 鍚庨潰娌℃湁鍙屾崲琛岀锛堢湡姝ｇ殑缁撴潫鏍囩鍚庨潰浼氭湁 `\n\n`锛?
/// - 鏍囩鍦ㄧ紦鍐插尯鏈熬锛堟祦寮忓鐞嗘椂闇€瑕佺瓑寰呮洿澶氬唴瀹癸級
///
/// # 鍙傛暟
/// - `buffer`: 瑕佹悳绱㈢殑瀛楃涓?
///
/// # 杩斿洖鍊?
/// - `Some(pos)`: 鐪熸鐨勭粨鏉熸爣绛剧殑璧峰浣嶇疆
/// - `None`: 娌℃湁鎵惧埌鐪熸鐨勭粨鏉熸爣绛?
fn find_real_thinking_end_tag(buffer: &str) -> Option<usize> {
    const TAG: &str = "</thinking>";
    let mut search_start = 0;

    while let Some(pos) = buffer[search_start..].find(TAG) {
        let absolute_pos = search_start + pos;

        // 妫€鏌ュ墠闈㈡槸鍚︽湁寮曠敤瀛楃
        let has_quote_before = absolute_pos > 0 && is_quote_char(buffer, absolute_pos - 1);

        // 妫€鏌ュ悗闈㈡槸鍚︽湁寮曠敤瀛楃
        let after_pos = absolute_pos + TAG.len();
        let has_quote_after = is_quote_char(buffer, after_pos);

        // 濡傛灉琚紩鐢ㄥ瓧绗﹀寘瑁癸紝璺宠繃
        if has_quote_before || has_quote_after {
            search_start = absolute_pos + 1;
            continue;
        }

        // 妫€鏌ュ悗闈㈢殑鍐呭
        let after_content = &buffer[after_pos..];

        // 濡傛灉鏍囩鍚庨潰鍐呭涓嶈冻浠ュ垽鏂槸鍚︽湁鍙屾崲琛岀锛岀瓑寰呮洿澶氬唴瀹?
        if after_content.len() < 2 {
            return None;
        }

        // 鐪熸鐨?thinking 缁撴潫鏍囩鍚庨潰浼氭湁鍙屾崲琛岀 `\n\n`
        if after_content.starts_with("\n\n") {
            return Some(absolute_pos);
        }

        // 涓嶆槸鍙屾崲琛岀锛岃烦杩囩户缁悳绱?
        search_start = absolute_pos + 1;
    }

    None
}

/// 鏌ユ壘缂撳啿鍖烘湯灏剧殑 thinking 缁撴潫鏍囩锛堝厑璁告湯灏惧彧鏈夌┖鐧藉瓧绗︼級
///
/// 鐢ㄤ簬鈥滆竟鐣屼簨浠垛€濆満鏅細渚嬪 thinking 缁撴潫鍚庣珛鍒昏繘鍏?tool_use锛屾垨娴佺粨鏉燂紝
/// 姝ゆ椂 `</thinking>` 鍚庨潰鍙兘娌℃湁 `\n\n`锛屼絾缁撴潫鏍囩渚濈劧搴旇璇嗗埆骞惰繃婊ゃ€?
///
/// 绾︽潫锛氬彧鏈夊綋 `</thinking>` 涔嬪悗鍏ㄩ儴閮芥槸绌虹櫧瀛楃鏃舵墠璁や负鏄粨鏉熸爣绛撅紝
/// 浠ラ伩鍏嶅湪 thinking 鍐呭涓彁鍒?`</thinking>`锛堥潪缁撴潫鏍囩锛夋椂璇垽銆?
fn find_real_thinking_end_tag_at_buffer_end(buffer: &str) -> Option<usize> {
    const TAG: &str = "</thinking>";
    let mut search_start = 0;

    while let Some(pos) = buffer[search_start..].find(TAG) {
        let absolute_pos = search_start + pos;

        // 妫€鏌ュ墠闈㈡槸鍚︽湁寮曠敤瀛楃
        let has_quote_before = absolute_pos > 0 && is_quote_char(buffer, absolute_pos - 1);

        // 妫€鏌ュ悗闈㈡槸鍚︽湁寮曠敤瀛楃
        let after_pos = absolute_pos + TAG.len();
        let has_quote_after = is_quote_char(buffer, after_pos);

        if has_quote_before || has_quote_after {
            search_start = absolute_pos + 1;
            continue;
        }

        // 鍙湁褰撴爣绛惧悗闈㈠叏閮ㄦ槸绌虹櫧瀛楃鏃舵墠璁ゅ畾涓虹粨鏉熸爣绛?
        if buffer[after_pos..].trim().is_empty() {
            return Some(absolute_pos);
        }

        search_start = absolute_pos + 1;
    }

    None
}

/// 鏌ユ壘鐪熸鐨?thinking 寮€濮嬫爣绛撅紙涓嶈寮曠敤瀛楃鍖呰９锛?
///
/// 涓?`find_real_thinking_end_tag` 绫讳技锛岃烦杩囪寮曠敤瀛楃鍖呰９鐨勫紑濮嬫爣绛俱€?
fn find_real_thinking_start_tag(buffer: &str) -> Option<usize> {
    const TAG: &str = "<thinking>";
    let mut search_start = 0;

    while let Some(pos) = buffer[search_start..].find(TAG) {
        let absolute_pos = search_start + pos;

        // 妫€鏌ュ墠闈㈡槸鍚︽湁寮曠敤瀛楃
        let has_quote_before = absolute_pos > 0 && is_quote_char(buffer, absolute_pos - 1);

        // 妫€鏌ュ悗闈㈡槸鍚︽湁寮曠敤瀛楃
        let after_pos = absolute_pos + TAG.len();
        let has_quote_after = is_quote_char(buffer, after_pos);

        // 濡傛灉涓嶈寮曠敤瀛楃鍖呰９锛屽垯鏄湡姝ｇ殑寮€濮嬫爣绛?
        if !has_quote_before && !has_quote_after {
            return Some(absolute_pos);
        }

        // 缁х画鎼滅储涓嬩竴涓尮閰?
        search_start = absolute_pos + 1;
    }

    None
}

/// SSE 浜嬩欢
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: String,
    pub data: serde_json::Value,
}

impl SseEvent {
    pub fn new(event: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            event: event.into(),
            data,
        }
    }

    /// 鏍煎紡鍖栦负 SSE 瀛楃涓?
    pub fn to_sse_string(&self) -> String {
        format!(
            "event: {}\ndata: {}\n\n",
            self.event,
            serde_json::to_string(&self.data).unwrap_or_default()
        )
    }
}

/// 鍐呭鍧楃姸鎬?
#[derive(Debug, Clone)]
struct BlockState {
    block_type: String,
    started: bool,
    stopped: bool,
}

impl BlockState {
    fn new(block_type: impl Into<String>) -> Self {
        Self {
            block_type: block_type.into(),
            started: false,
            stopped: false,
        }
    }
}

/// SSE 鐘舵€佺鐞嗗櫒
///
/// 纭繚 SSE 浜嬩欢搴忓垪绗﹀悎 Claude API 瑙勮寖锛?
/// 1. message_start 鍙兘鍑虹幇涓€娆?
/// 2. content_block 蹇呴』鍏?start 鍐?delta 鍐?stop
/// 3. message_delta 鍙兘鍑虹幇涓€娆★紝涓斿湪鎵€鏈?content_block_stop 涔嬪悗
/// 4. message_stop 鍦ㄦ渶鍚?
#[derive(Debug)]
pub struct SseStateManager {
    /// message_start 鏄惁宸插彂閫?
    message_started: bool,
    /// message_delta 鏄惁宸插彂閫?
    message_delta_sent: bool,
    /// 娲昏穬鐨勫唴瀹瑰潡鐘舵€?
    active_blocks: HashMap<i32, BlockState>,
    /// 娑堟伅鏄惁宸茬粨鏉?
    message_ended: bool,
    /// 涓嬩竴涓潡绱㈠紩
    next_block_index: i32,
    /// 褰撳墠 stop_reason
    stop_reason: Option<String>,
    /// 鏄惁鏈夊伐鍏疯皟鐢?
    has_tool_use: bool,
}

impl Default for SseStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SseStateManager {
    pub fn new() -> Self {
        Self {
            message_started: false,
            message_delta_sent: false,
            active_blocks: HashMap::new(),
            message_ended: false,
            next_block_index: 0,
            stop_reason: None,
            has_tool_use: false,
        }
    }

    /// 鍒ゆ柇鎸囧畾鍧楁槸鍚﹀浜庡彲鎺ユ敹 delta 鐨勬墦寮€鐘舵€?
    fn is_block_open_of_type(&self, index: i32, expected_type: &str) -> bool {
        self.active_blocks
            .get(&index)
            .is_some_and(|b| b.started && !b.stopped && b.block_type == expected_type)
    }

    /// 鑾峰彇涓嬩竴涓潡绱㈠紩
    pub fn next_block_index(&mut self) -> i32 {
        let index = self.next_block_index;
        self.next_block_index += 1;
        index
    }

    /// 璁板綍宸ュ叿璋冪敤
    pub fn set_has_tool_use(&mut self, has: bool) {
        self.has_tool_use = has;
    }

    /// 璁剧疆 stop_reason
    pub fn set_stop_reason(&mut self, reason: impl Into<String>) {
        self.stop_reason = Some(reason.into());
    }

    /// 妫€鏌ユ槸鍚﹀瓨鍦ㄩ潪 thinking 绫诲瀷鐨勫唴瀹瑰潡锛堝 text 鎴?tool_use锛?
    fn has_non_thinking_blocks(&self) -> bool {
        self.active_blocks
            .values()
            .any(|b| b.block_type != "thinking")
    }

    /// 鑾峰彇鏈€缁堢殑 stop_reason
    pub fn get_stop_reason(&self) -> String {
        if let Some(ref reason) = self.stop_reason {
            reason.clone()
        } else if self.has_tool_use {
            "tool_use".to_string()
        } else {
            "end_turn".to_string()
        }
    }

    /// 澶勭悊 message_start 浜嬩欢
    pub fn handle_message_start(&mut self, event: serde_json::Value) -> Option<SseEvent> {
        if self.message_started {
            tracing::debug!("璺宠繃閲嶅鐨?message_start 浜嬩欢");
            return None;
        }
        self.message_started = true;
        Some(SseEvent::new("message_start", event))
    }

    /// 澶勭悊 content_block_start 浜嬩欢
    pub fn handle_content_block_start(
        &mut self,
        index: i32,
        block_type: &str,
        data: serde_json::Value,
    ) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // 濡傛灉鏄?tool_use 鍧楋紝鍏堝叧闂箣鍓嶇殑鏂囨湰鍧?
        if block_type == "tool_use" {
            self.has_tool_use = true;
            for (block_index, block) in self.active_blocks.iter_mut() {
                if block.block_type == "text" && block.started && !block.stopped {
                    // 鑷姩鍙戦€?content_block_stop 鍏抽棴鏂囨湰鍧?
                    events.push(SseEvent::new(
                        "content_block_stop",
                        json!({
                            "type": "content_block_stop",
                            "index": block_index
                        }),
                    ));
                    block.stopped = true;
                }
            }
        }

        // 妫€鏌ュ潡鏄惁宸插瓨鍦?
        if let Some(block) = self.active_blocks.get_mut(&index) {
            if block.started {
                tracing::debug!("鍧?{} 宸插惎鍔紝璺宠繃閲嶅鐨?content_block_start", index);
                return events;
            }
            block.started = true;
        } else {
            let mut block = BlockState::new(block_type);
            block.started = true;
            self.active_blocks.insert(index, block);
        }

        events.push(SseEvent::new("content_block_start", data));
        events
    }

    /// 澶勭悊 content_block_delta 浜嬩欢
    pub fn handle_content_block_delta(
        &mut self,
        index: i32,
        data: serde_json::Value,
    ) -> Option<SseEvent> {
        // 纭繚鍧楀凡鍚姩
        if let Some(block) = self.active_blocks.get(&index) {
            if !block.started || block.stopped {
                tracing::warn!(
                    "鍧?{} 鐘舵€佸紓甯? started={}, stopped={}",
                    index,
                    block.started,
                    block.stopped
                );
                return None;
            }
        } else {
            // 鍧椾笉瀛樺湪锛屽彲鑳介渶瑕佸厛鍒涘缓
            tracing::warn!("鏀跺埌鏈煡鍧?{} 鐨?delta 浜嬩欢", index);
            return None;
        }

        Some(SseEvent::new("content_block_delta", data))
    }

    /// 澶勭悊 content_block_stop 浜嬩欢
    pub fn handle_content_block_stop(&mut self, index: i32) -> Option<SseEvent> {
        if let Some(block) = self.active_blocks.get_mut(&index) {
            if block.stopped {
                tracing::debug!("鍧?{} 宸插仠姝紝璺宠繃閲嶅鐨?content_block_stop", index);
                return None;
            }
            block.stopped = true;
            return Some(SseEvent::new(
                "content_block_stop",
                json!({
                    "type": "content_block_stop",
                    "index": index
                }),
            ));
        }
        None
    }

    /// 鐢熸垚鏈€缁堜簨浠跺簭鍒?
    pub fn generate_final_events(
        &mut self,
        input_tokens: i32,
        output_tokens: i32,
    ) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // 鍏抽棴鎵€鏈夋湭鍏抽棴鐨勫潡
        for (index, block) in self.active_blocks.iter_mut() {
            if block.started && !block.stopped {
                events.push(SseEvent::new(
                    "content_block_stop",
                    json!({
                        "type": "content_block_stop",
                        "index": index
                    }),
                ));
                block.stopped = true;
            }
        }

        // 鍙戦€?message_delta
        if !self.message_delta_sent {
            self.message_delta_sent = true;
            events.push(SseEvent::new(
                "message_delta",
                json!({
                    "type": "message_delta",
                    "delta": {
                        "stop_reason": self.get_stop_reason(),
                        "stop_sequence": null
                    },
                    "usage": {
                        "input_tokens": input_tokens,
                        "output_tokens": output_tokens
                    }
                }),
            ));
        }

        // 鍙戦€?message_stop
        if !self.message_ended {
            self.message_ended = true;
            events.push(SseEvent::new(
                "message_stop",
                json!({ "type": "message_stop" }),
            ));
        }

        events
    }
}

/// 娴佸鐞嗕笂涓嬫枃
pub struct StreamContext {
    /// SSE 鐘舵€佺鐞嗗櫒
    pub state_manager: SseStateManager,
    /// 璇锋眰鐨勬ā鍨嬪悕绉?
    pub model: String,
    /// 娑堟伅 ID
    pub message_id: String,
    /// 杈撳叆 tokens锛堜及绠楀€硷級
    pub input_tokens: i32,
    pub context_window: i32,
    /// 浠?contextUsageEvent 璁＄畻鐨勫疄闄呰緭鍏?tokens
    pub context_input_tokens: Option<i32>,
    /// 杈撳嚭 tokens 绱
    pub output_tokens: i32,
    /// 宸ュ叿鍧楃储寮曟槧灏?(tool_id -> block_index)
    pub tool_block_indices: HashMap<String, i32>,
    /// 宸ュ叿鍚嶇О鍙嶅悜鏄犲皠锛堢煭鍚嶇О 鈫?鍘熷鍚嶇О锛夛紝鐢ㄤ簬鍝嶅簲鏃惰繕鍘?
    pub tool_name_map: HashMap<String, String>,
    /// thinking 鏄惁鍚敤
    pub thinking_enabled: bool,
    /// thinking 鍐呭缂撳啿鍖?
    pub thinking_buffer: String,
    /// 鏄惁鍦?thinking 鍧楀唴
    pub in_thinking_block: bool,
    /// thinking 鍧楁槸鍚﹀凡鎻愬彇瀹屾垚
    pub thinking_extracted: bool,
    /// thinking 鍧楃储寮?
    pub thinking_block_index: Option<i32>,
    /// 鏂囨湰鍧楃储寮曪紙thinking 鍚敤鏃跺姩鎬佸垎閰嶏級
    pub text_block_index: Option<i32>,
    /// 鏄惁闇€瑕佸墺绂?thinking 鍐呭寮€澶寸殑鎹㈣绗?
    /// 妯″瀷杈撳嚭 `<thinking>\n` 鏃讹紝`\n` 鍙兘涓庢爣绛惧湪鍚屼竴 chunk 鎴栦笅涓€ chunk
    strip_thinking_leading_newline: bool,
}

impl StreamContext {
    /// 鍒涘缓鍚敤thinking鐨凷treamContext
    pub fn new_with_thinking(
        model: impl Into<String>,
        context_window: i32,
        input_tokens: i32,
        thinking_enabled: bool,
        tool_name_map: HashMap<String, String>,
    ) -> Self {
        Self {
            state_manager: SseStateManager::new(),
            model: model.into(),
            message_id: format!("msg_{}", Uuid::new_v4().to_string().replace('-', "")),
            input_tokens,
            context_window,
            context_input_tokens: None,
            output_tokens: 0,
            tool_block_indices: HashMap::new(),
            tool_name_map,
            thinking_enabled,
            thinking_buffer: String::new(),
            in_thinking_block: false,
            thinking_extracted: false,
            thinking_block_index: None,
            text_block_index: None,
            strip_thinking_leading_newline: false,
        }
    }

    /// 鐢熸垚 message_start 浜嬩欢
    pub fn create_message_start_event(&self) -> serde_json::Value {
        json!({
            "type": "message_start",
            "message": {
                "id": self.message_id,
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": self.model,
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {
                    "input_tokens": self.input_tokens,
                    "output_tokens": 1
                }
            }
        })
    }

    /// 鐢熸垚鍒濆浜嬩欢搴忓垪 (message_start + 鏂囨湰鍧?start)
    ///
    /// 褰?thinking 鍚敤鏃讹紝涓嶅湪鍒濆鍖栨椂鍒涘缓鏂囨湰鍧楋紝鑰屾槸绛夊埌瀹為檯鏀跺埌鍐呭鏃跺啀鍒涘缓銆?
    /// 杩欐牱鍙互纭繚 thinking 鍧楋紙绱㈠紩 0锛夊湪鏂囨湰鍧楋紙绱㈠紩 1锛変箣鍓嶃€?
    pub fn generate_initial_events(&mut self) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // message_start
        let msg_start = self.create_message_start_event();
        if let Some(event) = self.state_manager.handle_message_start(msg_start) {
            events.push(event);
        }

        // 濡傛灉鍚敤浜?thinking锛屼笉鍦ㄨ繖閲屽垱寤烘枃鏈潡
        // thinking 鍧楀拰鏂囨湰鍧椾細鍦?process_content_with_thinking 涓寜姝ｇ‘椤哄簭鍒涘缓
        if self.thinking_enabled {
            return events;
        }

        // 鍒涘缓鍒濆鏂囨湰鍧楋紙浠呭湪鏈惎鐢?thinking 鏃讹級
        let text_block_index = self.state_manager.next_block_index();
        self.text_block_index = Some(text_block_index);
        let text_block_events = self.state_manager.handle_content_block_start(
            text_block_index,
            "text",
            json!({
                "type": "content_block_start",
                "index": text_block_index,
                "content_block": {
                    "type": "text",
                    "text": ""
                }
            }),
        );
        events.extend(text_block_events);

        events
    }

    /// 澶勭悊 Kiro 浜嬩欢骞惰浆鎹负 Anthropic SSE 浜嬩欢
    pub fn process_kiro_event(&mut self, event: &Event) -> Vec<SseEvent> {
        match event {
            Event::AssistantResponse(resp) => self.process_assistant_response(&resp.content),
            Event::ToolUse(tool_use) => self.process_tool_use(tool_use),
            Event::ContextUsage(context_usage) => {
                // 浠庝笂涓嬫枃浣跨敤鐧惧垎姣旇绠楀疄闄呯殑 input_tokens
                let actual_input_tokens = (context_usage.context_usage_percentage
                    * (self.context_window as f64)
                    / 100.0) as i32;
                self.context_input_tokens = Some(actual_input_tokens);
                // 涓婁笅鏂囦娇鐢ㄩ噺杈惧埌 100% 鏃讹紝璁剧疆 stop_reason 涓?model_context_window_exceeded
                if context_usage.context_usage_percentage >= 100.0 {
                    self.state_manager
                        .set_stop_reason("model_context_window_exceeded");
                }
                tracing::debug!(
                    "鏀跺埌 contextUsageEvent: {}%, 璁＄畻 input_tokens: {}",
                    context_usage.context_usage_percentage,
                    actual_input_tokens
                );
                Vec::new()
            }
            Event::Error {
                error_code,
                error_message,
            } => {
                tracing::error!("鏀跺埌閿欒浜嬩欢: {} - {}", error_code, error_message);
                Vec::new()
            }
            Event::Exception {
                exception_type,
                message,
            } => {
                // 澶勭悊 ContentLengthExceededException
                if exception_type == "ContentLengthExceededException" {
                    self.state_manager.set_stop_reason("max_tokens");
                }
                tracing::warn!("鏀跺埌寮傚父浜嬩欢: {} - {}", exception_type, message);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// 澶勭悊鍔╂墜鍝嶅簲浜嬩欢
    fn process_assistant_response(&mut self, content: &str) -> Vec<SseEvent> {
        if content.is_empty() {
            return Vec::new();
        }

        // 浼扮畻 tokens
        self.output_tokens += estimate_tokens(content);

        // 濡傛灉鍚敤浜唗hinking锛岄渶瑕佸鐞唗hinking鍧?
        if self.thinking_enabled {
            return self.process_content_with_thinking(content);
        }

        // 闈?thinking 妯″紡鍚屾牱澶嶇敤缁熶竴鐨?text_delta 鍙戦€侀€昏緫锛?
        // 浠ヤ究鍦?tool_use 鑷姩鍏抽棴鏂囨湰鍧楀悗鑳藉鑷剤閲嶅缓鏂扮殑鏂囨湰鍧楋紝閬垮厤鈥滃悶瀛椻€濄€?
        self.create_text_delta_events(content)
    }

    /// 澶勭悊鍖呭惈thinking鍧楃殑鍐呭
    fn process_content_with_thinking(&mut self, content: &str) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // 灏嗗唴瀹规坊鍔犲埌缂撳啿鍖鸿繘琛屽鐞?
        self.thinking_buffer.push_str(content);

        loop {
            if !self.in_thinking_block && !self.thinking_extracted {
                // 鏌ユ壘 <thinking> 寮€濮嬫爣绛撅紙璺宠繃琚弽寮曞彿鍖呰９鐨勶級
                if let Some(start_pos) = find_real_thinking_start_tag(&self.thinking_buffer) {
                    // 鍙戦€?<thinking> 涔嬪墠鐨勫唴瀹逛綔涓?text_delta
                    // 娉ㄦ剰锛氬鏋滃墠闈㈠彧鏄┖鐧藉瓧绗︼紙濡?adaptive 妯″紡杩斿洖鐨?\n\n锛夛紝鍒欒烦杩囷紝
                    // 閬垮厤鍦?thinking 鍧椾箣鍓嶄骇鐢熸棤鎰忎箟鐨?text 鍧楀鑷村鎴风瑙ｆ瀽澶辫触
                    let before_thinking = self.thinking_buffer[..start_pos].to_string();
                    if !before_thinking.is_empty() && !before_thinking.trim().is_empty() {
                        events.extend(self.create_text_delta_events(&before_thinking));
                    }

                    // 杩涘叆 thinking 鍧?
                    self.in_thinking_block = true;
                    self.strip_thinking_leading_newline = true;
                    self.thinking_buffer =
                        self.thinking_buffer[start_pos + "<thinking>".len()..].to_string();

                    // 鍒涘缓 thinking 鍧楃殑 content_block_start 浜嬩欢
                    let thinking_index = self.state_manager.next_block_index();
                    self.thinking_block_index = Some(thinking_index);
                    let start_events = self.state_manager.handle_content_block_start(
                        thinking_index,
                        "thinking",
                        json!({
                            "type": "content_block_start",
                            "index": thinking_index,
                            "content_block": {
                                "type": "thinking",
                                "thinking": ""
                            }
                        }),
                    );
                    events.extend(start_events);
                } else {
                    // 娌℃湁鎵惧埌 <thinking>锛屾鏌ユ槸鍚﹀彲鑳芥槸閮ㄥ垎鏍囩
                    // 淇濈暀鍙兘鏄儴鍒嗘爣绛剧殑鍐呭
                    let target_len = self
                        .thinking_buffer
                        .len()
                        .saturating_sub("<thinking>".len());
                    let safe_len = find_char_boundary(&self.thinking_buffer, target_len);
                    if safe_len > 0 {
                        let safe_content = self.thinking_buffer[..safe_len].to_string();
                        // 濡傛灉 thinking 灏氭湭鎻愬彇锛屼笖瀹夊叏鍐呭鍙槸绌虹櫧瀛楃锛?
                        // 鍒欎笉鍙戦€佷负 text_delta锛岀户缁繚鐣欏湪缂撳啿鍖虹瓑寰呮洿澶氬唴瀹广€?
                        // 杩欓伩鍏嶄簡 4.6 妯″瀷涓?<thinking> 鏍囩璺ㄤ簨浠跺垎鍓叉椂锛?
                        // 鍓嶅绌虹櫧锛堝 "\n\n"锛夎閿欒鍦板垱寤轰负 text 鍧楋紝
                        // 瀵艰嚧 text 鍧楀厛浜?thinking 鍧楀嚭鐜扮殑闂銆?
                        if !safe_content.is_empty() && !safe_content.trim().is_empty() {
                            events.extend(self.create_text_delta_events(&safe_content));
                            self.thinking_buffer = self.thinking_buffer[safe_len..].to_string();
                        }
                    }
                    break;
                }
            } else if self.in_thinking_block {
                // 鍓ョ <thinking> 鏍囩鍚庣揣璺熺殑鎹㈣绗︼紙鍙兘璺?chunk锛?
                if self.strip_thinking_leading_newline {
                    if self.thinking_buffer.starts_with('\n') {
                        self.thinking_buffer = self.thinking_buffer[1..].to_string();
                        self.strip_thinking_leading_newline = false;
                    } else if !self.thinking_buffer.is_empty() {
                        // buffer 闈炵┖浣嗕笉浠?\n 寮€澶达紝涓嶅啀闇€瑕佸墺绂?
                        self.strip_thinking_leading_newline = false;
                    }
                    // buffer 涓虹┖鏃朵繚鐣欐爣蹇楋紝绛夊緟涓嬩竴涓?chunk
                }

                // 鍦?thinking 鍧楀唴锛屾煡鎵?</thinking> 缁撴潫鏍囩锛堣烦杩囪鍙嶅紩鍙峰寘瑁圭殑锛?
                if let Some(end_pos) = find_real_thinking_end_tag(&self.thinking_buffer) {
                    // 鎻愬彇 thinking 鍐呭
                    let thinking_content = self.thinking_buffer[..end_pos].to_string();
                    if !thinking_content.is_empty() {
                        if let Some(thinking_index) = self.thinking_block_index {
                            events.push(
                                self.create_thinking_delta_event(thinking_index, &thinking_content),
                            );
                        }
                    }

                    // 缁撴潫 thinking 鍧?
                    self.in_thinking_block = false;
                    self.thinking_extracted = true;

                    // 鍙戦€佺┖鐨?thinking_delta 浜嬩欢锛岀劧鍚庡彂閫?content_block_stop 浜嬩欢
                    if let Some(thinking_index) = self.thinking_block_index {
                        // 鍏堝彂閫佺┖鐨?thinking_delta
                        events.push(self.create_thinking_delta_event(thinking_index, ""));
                        // 鍐嶅彂閫?content_block_stop
                        if let Some(stop_event) =
                            self.state_manager.handle_content_block_stop(thinking_index)
                        {
                            events.push(stop_event);
                        }
                    }

                    // 鍓ョ `</thinking>\n\n`锛坒ind_real_thinking_end_tag 宸茬‘璁?\n\n 瀛樺湪锛?
                    self.thinking_buffer =
                        self.thinking_buffer[end_pos + "</thinking>\n\n".len()..].to_string();
                } else {
                    // 娌℃湁鎵惧埌缁撴潫鏍囩锛屽彂閫佸綋鍓嶇紦鍐插尯鍐呭浣滀负 thinking_delta銆?
                    // 淇濈暀鏈熬鍙兘鏄儴鍒?`</thinking>\n\n` 鐨勫唴瀹癸細
                    // find_real_thinking_end_tag 瑕佹眰鏍囩鍚庢湁 `\n\n` 鎵嶈繑鍥?Some锛?
                    // 鍥犳淇濈暀鍖哄繀椤昏鐩?`</thinking>\n\n` 鐨勫畬鏁撮暱搴︼紙13 瀛楄妭锛夛紝
                    // 鍚﹀垯褰?`</thinking>` 宸插湪 buffer 浣?`\n\n` 灏氭湭鍒拌揪鏃讹紝
                    // 鏍囩鐨勫墠鍑犱釜瀛楃浼氳閿欒鍦颁綔涓?thinking_delta 鍙戝嚭銆?
                    let target_len = self
                        .thinking_buffer
                        .len()
                        .saturating_sub("</thinking>\n\n".len());
                    let safe_len = find_char_boundary(&self.thinking_buffer, target_len);
                    if safe_len > 0 {
                        let safe_content = self.thinking_buffer[..safe_len].to_string();
                        if !safe_content.is_empty() {
                            if let Some(thinking_index) = self.thinking_block_index {
                                events.push(
                                    self.create_thinking_delta_event(thinking_index, &safe_content),
                                );
                            }
                        }
                        self.thinking_buffer = self.thinking_buffer[safe_len..].to_string();
                    }
                    break;
                }
            } else {
                // thinking 宸叉彁鍙栧畬鎴愶紝鍓╀綑鍐呭浣滀负 text_delta
                if !self.thinking_buffer.is_empty() {
                    let remaining = self.thinking_buffer.clone();
                    self.thinking_buffer.clear();
                    events.extend(self.create_text_delta_events(&remaining));
                }
                break;
            }
        }

        events
    }

    /// 鍒涘缓 text_delta 浜嬩欢
    ///
    /// 濡傛灉鏂囨湰鍧楀皻鏈垱寤猴紝浼氬厛鍒涘缓鏂囨湰鍧椼€?
    /// 褰撳彂鐢?tool_use 鏃讹紝鐘舵€佹満浼氳嚜鍔ㄥ叧闂綋鍓嶆枃鏈潡锛涘悗缁枃鏈細鑷姩鍒涘缓鏂扮殑鏂囨湰鍧楃户缁緭鍑恒€?
    ///
    /// 杩斿洖鍊煎寘鍚彲鑳界殑 content_block_start 浜嬩欢鍜?content_block_delta 浜嬩欢銆?
    fn create_text_delta_events(&mut self, text: &str) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // 濡傛灉褰撳墠 text_block_index 鎸囧悜鐨勫潡宸茬粡琚叧闂紙渚嬪 tool_use 寮€濮嬫椂鑷姩 stop锛夛紝
        // 鍒欎涪寮冭绱㈠紩骞跺垱寤烘柊鐨勬枃鏈潡缁х画杈撳嚭锛岄伩鍏?delta 琚姸鎬佹満鎷掔粷瀵艰嚧鈥滃悶瀛椻€濄€?
        if let Some(idx) = self.text_block_index {
            if !self.state_manager.is_block_open_of_type(idx, "text") {
                self.text_block_index = None;
            }
        }

        // 鑾峰彇鎴栧垱寤烘枃鏈潡绱㈠紩
        let text_index = if let Some(idx) = self.text_block_index {
            idx
        } else {
            // 鏂囨湰鍧楀皻鏈垱寤猴紝闇€瑕佸厛鍒涘缓
            let idx = self.state_manager.next_block_index();
            self.text_block_index = Some(idx);

            // 鍙戦€?content_block_start 浜嬩欢
            let start_events = self.state_manager.handle_content_block_start(
                idx,
                "text",
                json!({
                    "type": "content_block_start",
                    "index": idx,
                    "content_block": {
                        "type": "text",
                        "text": ""
                    }
                }),
            );
            events.extend(start_events);
            idx
        };

        // 鍙戦€?content_block_delta 浜嬩欢
        if let Some(delta_event) = self.state_manager.handle_content_block_delta(
            text_index,
            json!({
                "type": "content_block_delta",
                "index": text_index,
                "delta": {
                    "type": "text_delta",
                    "text": text
                }
            }),
        ) {
            events.push(delta_event);
        }

        events
    }

    /// 鍒涘缓 thinking_delta 浜嬩欢
    fn create_thinking_delta_event(&self, index: i32, thinking: &str) -> SseEvent {
        SseEvent::new(
            "content_block_delta",
            json!({
                "type": "content_block_delta",
                "index": index,
                "delta": {
                    "type": "thinking_delta",
                    "thinking": thinking
                }
            }),
        )
    }

    /// 澶勭悊宸ュ叿浣跨敤浜嬩欢
    fn process_tool_use(
        &mut self,
        tool_use: &crate::kiro::model::events::ToolUseEvent,
    ) -> Vec<SseEvent> {
        let mut events = Vec::new();

        self.state_manager.set_has_tool_use(true);

        // tool_use 蹇呴』鍙戠敓鍦?thinking 缁撴潫涔嬪悗銆?
        // 浣嗗綋 `</thinking>` 鍚庨潰娌℃湁 `\n\n`锛堜緥濡傜揣璺?tool_use 鎴栨祦缁撴潫锛夋椂锛?
        // thinking 缁撴潫鏍囩浼氭粸鐣欏湪 thinking_buffer锛屽鑷村悗缁?flush 鏃舵妸 `</thinking>` 褰撲綔鍐呭杈撳嚭銆?
        // 杩欓噷鍦ㄥ紑濮?tool_use block 鍓嶅仛涓€娆♀€滆竟鐣屽満鏅€濈殑缁撴潫鏍囩璇嗗埆涓庤繃婊ゃ€?
        if self.thinking_enabled && self.in_thinking_block {
            if let Some(end_pos) = find_real_thinking_end_tag_at_buffer_end(&self.thinking_buffer) {
                let thinking_content = self.thinking_buffer[..end_pos].to_string();
                if !thinking_content.is_empty() {
                    if let Some(thinking_index) = self.thinking_block_index {
                        events.push(
                            self.create_thinking_delta_event(thinking_index, &thinking_content),
                        );
                    }
                }

                // 缁撴潫 thinking 鍧?
                self.in_thinking_block = false;
                self.thinking_extracted = true;

                if let Some(thinking_index) = self.thinking_block_index {
                    // 鍏堝彂閫佺┖鐨?thinking_delta
                    events.push(self.create_thinking_delta_event(thinking_index, ""));
                    // 鍐嶅彂閫?content_block_stop
                    if let Some(stop_event) =
                        self.state_manager.handle_content_block_stop(thinking_index)
                    {
                        events.push(stop_event);
                    }
                }

                // 鎶婄粨鏉熸爣绛惧悗鐨勫唴瀹瑰綋浣滄櫘閫氭枃鏈紙閫氬父涓虹┖鎴栫┖鐧斤級
                let after_pos = end_pos + "</thinking>".len();
                let remaining = self.thinking_buffer[after_pos..].trim_start().to_string();
                self.thinking_buffer.clear();
                if !remaining.is_empty() {
                    events.extend(self.create_text_delta_events(&remaining));
                }
            }
        }

        // thinking 妯″紡涓嬶紝process_content_with_thinking 鍙兘浼氫负浜嗘帰娴?`<thinking>` 鑰屾殏瀛樹竴灏忔灏鹃儴鏂囨湰銆?
        // 濡傛灉姝ゆ椂鐩存帴寮€濮?tool_use锛岀姸鎬佹満浼氳嚜鍔ㄥ叧闂?text block锛屽鑷磋繖娈?寰呰緭鍑烘枃鏈?鐪嬭捣鏉ヨ tool_use 鍚炴帀銆?
        // 绾︽潫锛氬彧鍦ㄥ皻鏈繘鍏?thinking block銆佷笖 thinking 灏氭湭琚彁鍙栨椂锛屽皢缂撳啿鍖哄綋浣滄櫘閫氭枃鏈?flush銆?
        if self.thinking_enabled
            && !self.in_thinking_block
            && !self.thinking_extracted
            && !self.thinking_buffer.is_empty()
        {
            let buffered = std::mem::take(&mut self.thinking_buffer);
            events.extend(self.create_text_delta_events(&buffered));
        }

        // 鑾峰彇鎴栧垎閰嶅潡绱㈠紩
        let block_index = if let Some(&idx) = self.tool_block_indices.get(&tool_use.tool_use_id) {
            idx
        } else {
            let idx = self.state_manager.next_block_index();
            self.tool_block_indices
                .insert(tool_use.tool_use_id.clone(), idx);
            idx
        };

        // 杩樺師宸ュ叿鍚嶇О锛堝鏋滄湁鏄犲皠锛?
        let original_name = self
            .tool_name_map
            .get(&tool_use.name)
            .cloned()
            .unwrap_or_else(|| tool_use.name.clone());

        // 鍙戦€?content_block_start
        let start_events = self.state_manager.handle_content_block_start(
            block_index,
            "tool_use",
            json!({
                "type": "content_block_start",
                "index": block_index,
                "content_block": {
                    "type": "tool_use",
                    "id": tool_use.tool_use_id,
                    "name": original_name,
                    "input": {}
                }
            }),
        );
        events.extend(start_events);

        // 鍙戦€佸弬鏁板閲?(ToolUseEvent.input 鏄?String 绫诲瀷)
        if !tool_use.input.is_empty() {
            self.output_tokens += (tool_use.input.len() as i32 + 3) / 4; // 浼扮畻 token

            if let Some(delta_event) = self.state_manager.handle_content_block_delta(
                block_index,
                json!({
                    "type": "content_block_delta",
                    "index": block_index,
                    "delta": {
                        "type": "input_json_delta",
                        "partial_json": tool_use.input
                    }
                }),
            ) {
                events.push(delta_event);
            }
        }

        // 濡傛灉鏄畬鏁寸殑宸ュ叿璋冪敤锛坰top=true锛夛紝鍙戦€?content_block_stop
        if tool_use.stop {
            if let Some(stop_event) = self.state_manager.handle_content_block_stop(block_index) {
                events.push(stop_event);
            }
        }

        events
    }

    /// 鐢熸垚鏈€缁堜簨浠跺簭鍒?
    pub fn generate_final_events(&mut self) -> Vec<SseEvent> {
        let mut events = Vec::new();

        // Flush thinking_buffer 涓殑鍓╀綑鍐呭
        if self.thinking_enabled && !self.thinking_buffer.is_empty() {
            if self.in_thinking_block {
                // 鏈熬鍙兘娈嬬暀 `</thinking>`锛堜緥濡傜揣璺?tool_use 鎴栨祦缁撴潫锛夛紝闇€瑕佸湪 flush 鏃惰繃婊ゆ帀缁撴潫鏍囩銆?
                if let Some(end_pos) =
                    find_real_thinking_end_tag_at_buffer_end(&self.thinking_buffer)
                {
                    let thinking_content = self.thinking_buffer[..end_pos].to_string();
                    if !thinking_content.is_empty() {
                        if let Some(thinking_index) = self.thinking_block_index {
                            events.push(
                                self.create_thinking_delta_event(thinking_index, &thinking_content),
                            );
                        }
                    }

                    // 鍏抽棴 thinking 鍧楋細鍏堝彂閫佺┖鐨?thinking_delta锛屽啀鍙戦€?content_block_stop
                    if let Some(thinking_index) = self.thinking_block_index {
                        events.push(self.create_thinking_delta_event(thinking_index, ""));
                        if let Some(stop_event) =
                            self.state_manager.handle_content_block_stop(thinking_index)
                        {
                            events.push(stop_event);
                        }
                    }

                    // 鎶婄粨鏉熸爣绛惧悗鐨勫唴瀹瑰綋浣滄櫘閫氭枃鏈紙閫氬父涓虹┖鎴栫┖鐧斤級
                    let after_pos = end_pos + "</thinking>".len();
                    let remaining = self.thinking_buffer[after_pos..].trim_start().to_string();
                    self.thinking_buffer.clear();
                    self.in_thinking_block = false;
                    self.thinking_extracted = true;
                    if !remaining.is_empty() {
                        events.extend(self.create_text_delta_events(&remaining));
                    }
                } else {
                    // 濡傛灉杩樺湪 thinking 鍧楀唴锛屽彂閫佸墿浣欏唴瀹逛綔涓?thinking_delta
                    if let Some(thinking_index) = self.thinking_block_index {
                        events.push(
                            self.create_thinking_delta_event(thinking_index, &self.thinking_buffer),
                        );
                    }
                    // 鍏抽棴 thinking 鍧楋細鍏堝彂閫佺┖鐨?thinking_delta锛屽啀鍙戦€?content_block_stop
                    if let Some(thinking_index) = self.thinking_block_index {
                        // 鍏堝彂閫佺┖鐨?thinking_delta
                        events.push(self.create_thinking_delta_event(thinking_index, ""));
                        // 鍐嶅彂閫?content_block_stop
                        if let Some(stop_event) =
                            self.state_manager.handle_content_block_stop(thinking_index)
                        {
                            events.push(stop_event);
                        }
                    }
                }
            } else {
                // 鍚﹀垯鍙戦€佸墿浣欏唴瀹逛綔涓?text_delta
                let buffer_content = self.thinking_buffer.clone();
                events.extend(self.create_text_delta_events(&buffer_content));
            }
            self.thinking_buffer.clear();
        }

        // 濡傛灉鏁翠釜娴佷腑鍙骇鐢熶簡 thinking 鍧楋紝娌℃湁 text 涔熸病鏈?tool_use锛?
        // 鍒欒缃?stop_reason 涓?max_tokens锛堣〃绀烘ā鍨嬭€楀敖浜?token 棰勭畻鍦ㄦ€濊€冧笂锛夛紝
        // 骞惰ˉ鍙戜竴濂楀畬鏁寸殑 text 浜嬩欢锛堝唴瀹逛负涓€涓┖鏍硷級锛岀‘淇?content 鏁扮粍涓湁 text 鍧?
        if self.thinking_enabled
            && self.thinking_block_index.is_some()
            && !self.state_manager.has_non_thinking_blocks()
        {
            self.state_manager.set_stop_reason("max_tokens");
            events.extend(self.create_text_delta_events(" "));
        }

        // 浣跨敤浠?contextUsageEvent 璁＄畻鐨?input_tokens锛屽鏋滄病鏈夊垯浣跨敤浼扮畻鍊?
        let final_input_tokens = self.context_input_tokens.unwrap_or(self.input_tokens);

        // 鐢熸垚鏈€缁堜簨浠?
        events.extend(
            self.state_manager
                .generate_final_events(final_input_tokens, self.output_tokens),
        );
        events
    }
}

/// 缂撳啿娴佸鐞嗕笂涓嬫枃 - 鐢ㄤ簬 /cc/v1/messages 娴佸紡璇锋眰
///
/// 涓?`StreamContext` 涓嶅悓锛屾涓婁笅鏂囦細缂撳啿鎵€鏈変簨浠剁洿鍒版祦缁撴潫锛?
/// 鐒跺悗鐢ㄤ粠 `contextUsageEvent` 璁＄畻鐨勬纭?`input_tokens` 鏇存 `message_start` 浜嬩欢銆?
///
/// 宸ヤ綔娴佺▼锛?
/// 1. 浣跨敤 `StreamContext` 姝ｅ父澶勭悊鎵€鏈?Kiro 浜嬩欢
/// 2. 鎶婄敓鎴愮殑 SSE 浜嬩欢缂撳瓨璧锋潵锛堣€屼笉鏄珛鍗冲彂閫侊級
/// 3. 娴佺粨鏉熸椂锛屾壘鍒?`message_start` 浜嬩欢骞舵洿鏂板叾 `input_tokens`
/// 4. 涓€娆℃€ц繑鍥炴墍鏈変簨浠?
pub struct BufferedStreamContext {
    /// 鍐呴儴娴佸鐞嗕笂涓嬫枃锛堝鐢ㄧ幇鏈夌殑浜嬩欢澶勭悊閫昏緫锛?
    inner: StreamContext,
    /// 缂撳啿鐨勬墍鏈変簨浠讹紙鍖呮嫭 message_start銆乧ontent_block_start 绛夛級
    event_buffer: Vec<SseEvent>,
    /// 浼扮畻鐨?input_tokens锛堢敤浜庡洖閫€锛?
    estimated_input_tokens: i32,
    /// 鏄惁宸茬粡鐢熸垚浜嗗垵濮嬩簨浠?
    initial_events_generated: bool,
}

impl BufferedStreamContext {
    /// 鍒涘缓缂撳啿娴佷笂涓嬫枃
    pub fn new(
        model: impl Into<String>,
        context_window: i32,
        estimated_input_tokens: i32,
        thinking_enabled: bool,
        tool_name_map: HashMap<String, String>,
    ) -> Self {
        let inner = StreamContext::new_with_thinking(
            model,
            context_window,
            estimated_input_tokens,
            thinking_enabled,
            tool_name_map,
        );
        Self {
            inner,
            event_buffer: Vec::new(),
            estimated_input_tokens,
            initial_events_generated: false,
        }
    }

    /// 澶勭悊 Kiro 浜嬩欢骞剁紦鍐茬粨鏋?
    ///
    /// 澶嶇敤 StreamContext 鐨勪簨浠跺鐞嗛€昏緫锛屼絾鎶婄粨鏋滅紦瀛樿€屼笉鏄珛鍗冲彂閫併€?
    pub fn process_and_buffer(&mut self, event: &crate::kiro::model::events::Event) {
        // 棣栨澶勭悊浜嬩欢鏃讹紝鍏堢敓鎴愬垵濮嬩簨浠讹紙message_start 绛夛級
        if !self.initial_events_generated {
            let initial_events = self.inner.generate_initial_events();
            self.event_buffer.extend(initial_events);
            self.initial_events_generated = true;
        }

        // 澶勭悊浜嬩欢骞剁紦鍐茬粨鏋?
        let events = self.inner.process_kiro_event(event);
        self.event_buffer.extend(events);
    }

    /// 瀹屾垚娴佸鐞嗗苟杩斿洖鎵€鏈変簨浠?
    ///
    /// 姝ゆ柟娉曚細锛?
    /// 1. 鐢熸垚鏈€缁堜簨浠讹紙message_delta, message_stop锛?
    /// 2. 鐢ㄦ纭殑 input_tokens 鏇存 message_start 浜嬩欢
    /// 3. 杩斿洖鎵€鏈夌紦鍐茬殑浜嬩欢
    pub fn finish_and_get_all_events(&mut self) -> Vec<SseEvent> {
        // 濡傛灉浠庢湭澶勭悊杩囦簨浠讹紝涔熻鐢熸垚鍒濆浜嬩欢
        if !self.initial_events_generated {
            let initial_events = self.inner.generate_initial_events();
            self.event_buffer.extend(initial_events);
            self.initial_events_generated = true;
        }

        // 鐢熸垚鏈€缁堜簨浠?
        let final_events = self.inner.generate_final_events();
        self.event_buffer.extend(final_events);

        // 鑾峰彇姝ｇ‘鐨?input_tokens
        let final_input_tokens = self
            .inner
            .context_input_tokens
            .unwrap_or(self.estimated_input_tokens);

        // 鏇存 message_start 浜嬩欢涓殑 input_tokens
        for event in &mut self.event_buffer {
            if event.event == "message_start" {
                if let Some(message) = event.data.get_mut("message") {
                    if let Some(usage) = message.get_mut("usage") {
                        usage["input_tokens"] = serde_json::json!(final_input_tokens);
                    }
                }
            }
        }

        std::mem::take(&mut self.event_buffer)
    }
}

/// 绠€鍗曠殑 token 浼扮畻
fn estimate_tokens(text: &str) -> i32 {
    let chars: Vec<char> = text.chars().collect();
    let mut chinese_count = 0;
    let mut other_count = 0;

    for c in &chars {
        if *c >= '\u{4E00}' && *c <= '\u{9FFF}' {
            chinese_count += 1;
        } else {
            other_count += 1;
        }
    }

    // 涓枃绾?1.5 瀛楃/token锛岃嫳鏂囩害 4 瀛楃/token
    let chinese_tokens = (chinese_count * 2 + 2) / 3;
    let other_tokens = (other_count + 3) / 4;

    (chinese_tokens + other_tokens).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_format() {
        let event = SseEvent::new("message_start", json!({"type": "message_start"}));
        let sse_str = event.to_sse_string();

        assert!(sse_str.starts_with("event: message_start\n"));
        assert!(sse_str.contains("data: "));
        assert!(sse_str.ends_with("\n\n"));
    }

    #[test]
    fn test_sse_state_manager_message_start() {
        let mut manager = SseStateManager::new();

        // 绗竴娆″簲璇ユ垚鍔?
        let event = manager.handle_message_start(json!({"type": "message_start"}));
        assert!(event.is_some());

        // 绗簩娆″簲璇ヨ璺宠繃
        let event = manager.handle_message_start(json!({"type": "message_start"}));
        assert!(event.is_none());
    }

    #[test]
    fn test_sse_state_manager_block_lifecycle() {
        let mut manager = SseStateManager::new();

        // 鍒涘缓鍧?
        let events = manager.handle_content_block_start(0, "text", json!({}));
        assert_eq!(events.len(), 1);

        // delta
        let event = manager.handle_content_block_delta(0, json!({}));
        assert!(event.is_some());

        // stop
        let event = manager.handle_content_block_stop(0);
        assert!(event.is_some());

        // 閲嶅 stop 搴旇琚烦杩?
        let event = manager.handle_content_block_stop(0);
        assert!(event.is_none());
    }

    #[test]
    fn test_tool_name_reverse_mapping_in_stream() {
        use crate::kiro::model::events::ToolUseEvent;

        let mut map = HashMap::new();
        map.insert(
            "short_abc12345".to_string(),
            "mcp__very_long_original_tool_name".to_string(),
        );

        let mut ctx = StreamContext::new_with_thinking("test-model", 200_000, 1, false, map);
        let _ = ctx.generate_initial_events();

        // 妯℃嫙 Kiro 杩斿洖鐭悕绉扮殑 tool_use
        let tool_event = Event::ToolUse(ToolUseEvent {
            name: "short_abc12345".to_string(),
            tool_use_id: "toolu_01".to_string(),
            input: r#"{"key":"value"}"#.to_string(),
            stop: true,
        });

        let events = ctx.process_kiro_event(&tool_event);

        // content_block_start 涓殑 name 搴旇鏄師濮嬮暱鍚嶇О
        let start_event = events
            .iter()
            .find(|e| e.event == "content_block_start")
            .unwrap();
        assert_eq!(
            start_event.data["content_block"]["name"], "mcp__very_long_original_tool_name",
            "搴旇繕鍘熶负鍘熷宸ュ叿鍚嶇О"
        );
    }

    #[test]
    fn test_text_delta_after_tool_use_restarts_text_block() {
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, false, HashMap::new());

        let initial_events = ctx.generate_initial_events();
        assert!(
            initial_events
                .iter()
                .any(|e| e.event == "content_block_start"
                    && e.data["content_block"]["type"] == "text")
        );

        let initial_text_index = ctx
            .text_block_index
            .expect("initial text block index should exist");

        // tool_use 寮€濮嬩細鑷姩鍏抽棴鐜版湁 text block
        let tool_events = ctx.process_tool_use(&crate::kiro::model::events::ToolUseEvent {
            name: "test_tool".to_string(),
            tool_use_id: "tool_1".to_string(),
            input: "{}".to_string(),
            stop: false,
        });
        assert!(
            tool_events.iter().any(|e| {
                e.event == "content_block_stop"
                    && e.data["index"].as_i64() == Some(initial_text_index as i64)
            }),
            "tool_use should stop the previous text block"
        );

        // 涔嬪悗鍐嶆潵鏂囨湰澧為噺锛屽簲鑷姩鍒涘缓鏂扮殑 text block 鑰屼笉鏄線宸?stop 鐨勫潡閲屽啓 delta
        let text_events = ctx.process_assistant_response("hello");
        let new_text_start_index = text_events.iter().find_map(|e| {
            if e.event == "content_block_start" && e.data["content_block"]["type"] == "text" {
                e.data["index"].as_i64()
            } else {
                None
            }
        });
        assert!(
            new_text_start_index.is_some(),
            "should start a new text block"
        );
        assert_ne!(
            new_text_start_index.unwrap(),
            initial_text_index as i64,
            "new text block index should differ from the stopped one"
        );
        assert!(
            text_events.iter().any(|e| {
                e.event == "content_block_delta"
                    && e.data["delta"]["type"] == "text_delta"
                    && e.data["delta"]["text"] == "hello"
            }),
            "should emit text_delta after restarting text block"
        );
    }

    #[test]
    fn test_tool_use_flushes_pending_thinking_buffer_text_before_tool_block() {
        // thinking 妯″紡涓嬶紝鐭枃鏈彲鑳借鏆傚瓨鍦?thinking_buffer 浠ョ瓑寰?`<thinking>` 鐨勮法 chunk 鍖归厤銆?
        // 褰撶揣鎺ョ潃鍑虹幇 tool_use 鏃讹紝搴斿厛 flush 杩欐鏂囨湰锛屽啀寮€濮?tool_use block銆?
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        // 涓ゆ鐭枃鏈紙鍚?2 涓腑鏂囧瓧绗︼級锛屾€婚暱搴︿粛鍙兘涓嶈冻浠ユ弧瓒?safe_len>0 鐨勮緭鍑烘潯浠讹紝
        // 鍥犺€屼細鐣欏湪 thinking_buffer 涓瓑寰呭悗缁?chunk銆?
        let ev1 = ctx.process_assistant_response("鏈変慨");
        assert!(
            ev1.iter().all(|e| e.event != "content_block_delta"),
            "short prefix should be buffered under thinking mode"
        );
        let ev2 = ctx.process_assistant_response("鏀癸細");
        assert!(
            ev2.iter().all(|e| e.event != "content_block_delta"),
            "short prefix should still be buffered under thinking mode"
        );

        let events = ctx.process_tool_use(&crate::kiro::model::events::ToolUseEvent {
            name: "Write".to_string(),
            tool_use_id: "tool_1".to_string(),
            input: "{}".to_string(),
            stop: false,
        });

        let text_start_index = events.iter().find_map(|e| {
            if e.event == "content_block_start" && e.data["content_block"]["type"] == "text" {
                e.data["index"].as_i64()
            } else {
                None
            }
        });
        let pos_text_delta = events.iter().position(|e| {
            e.event == "content_block_delta" && e.data["delta"]["type"] == "text_delta"
        });
        let pos_text_stop = text_start_index.and_then(|idx| {
            events.iter().position(|e| {
                e.event == "content_block_stop" && e.data["index"].as_i64() == Some(idx)
            })
        });
        let pos_tool_start = events.iter().position(|e| {
            e.event == "content_block_start" && e.data["content_block"]["type"] == "tool_use"
        });

        assert!(
            text_start_index.is_some(),
            "should start a text block to flush buffered text"
        );
        assert!(
            pos_text_delta.is_some(),
            "should flush buffered text as text_delta"
        );
        assert!(
            pos_text_stop.is_some(),
            "should stop text block before tool_use block starts"
        );
        assert!(pos_tool_start.is_some(), "should start tool_use block");

        let pos_text_delta = pos_text_delta.unwrap();
        let pos_text_stop = pos_text_stop.unwrap();
        let pos_tool_start = pos_tool_start.unwrap();

        assert!(
            pos_text_delta < pos_text_stop && pos_text_stop < pos_tool_start,
            "ordering should be: text_delta -> text_stop -> tool_use_start"
        );

        assert!(
            events.iter().any(|e| {
                e.event == "content_block_delta"
                    && e.data["delta"]["type"] == "text_delta"
                    && e.data["delta"]["text"] == "鏈変慨鏀癸細"
            }),
            "flushed text should equal the buffered prefix"
        );
    }

    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("Hello") > 0);
        assert!(estimate_tokens("浣犲ソ") > 0);
        assert!(estimate_tokens("Hello 浣犲ソ") > 0);
    }

    #[test]
    fn test_find_real_thinking_start_tag_basic() {
        // 鍩烘湰鎯呭喌锛氭甯哥殑寮€濮嬫爣绛?
        assert_eq!(find_real_thinking_start_tag("<thinking>"), Some(0));
        assert_eq!(find_real_thinking_start_tag("prefix<thinking>"), Some(6));
    }

    #[test]
    fn test_find_real_thinking_start_tag_with_backticks() {
        // 琚弽寮曞彿鍖呰９鐨勫簲璇ヨ璺宠繃
        assert_eq!(find_real_thinking_start_tag("`<thinking>`"), None);
        assert_eq!(find_real_thinking_start_tag("use `<thinking>` tag"), None);

        // 鍏堟湁琚寘瑁圭殑锛屽悗鏈夌湡姝ｇ殑寮€濮嬫爣绛?
        assert_eq!(
            find_real_thinking_start_tag("about `<thinking>` tag<thinking>content"),
            Some(22)
        );
    }

    #[test]
    fn test_find_real_thinking_start_tag_with_quotes() {
        // 琚弻寮曞彿鍖呰９鐨勫簲璇ヨ璺宠繃
        assert_eq!(find_real_thinking_start_tag("\"<thinking>\""), None);
        assert_eq!(find_real_thinking_start_tag("the \"<thinking>\" tag"), None);

        // 琚崟寮曞彿鍖呰９鐨勫簲璇ヨ璺宠繃
        assert_eq!(find_real_thinking_start_tag("'<thinking>'"), None);

        // 娣峰悎鎯呭喌
        assert_eq!(
            find_real_thinking_start_tag("about \"<thinking>\" and '<thinking>' then<thinking>"),
            Some(40)
        );
    }

    #[test]
    fn test_find_real_thinking_end_tag_basic() {
        // 鍩烘湰鎯呭喌锛氭甯哥殑缁撴潫鏍囩鍚庨潰鏈夊弻鎹㈣绗?
        assert_eq!(find_real_thinking_end_tag("</thinking>\n\n"), Some(0));
        assert_eq!(
            find_real_thinking_end_tag("content</thinking>\n\n"),
            Some(7)
        );
        assert_eq!(
            find_real_thinking_end_tag("some text</thinking>\n\nmore text"),
            Some(9)
        );

        // 娌℃湁鍙屾崲琛岀鐨勬儏鍐?
        assert_eq!(find_real_thinking_end_tag("</thinking>"), None);
        assert_eq!(find_real_thinking_end_tag("</thinking>\n"), None);
        assert_eq!(find_real_thinking_end_tag("</thinking> more"), None);
    }

    #[test]
    fn test_find_real_thinking_end_tag_with_backticks() {
        // 琚弽寮曞彿鍖呰９鐨勫簲璇ヨ璺宠繃
        assert_eq!(find_real_thinking_end_tag("`</thinking>`\n\n"), None);
        assert_eq!(
            find_real_thinking_end_tag("mention `</thinking>` in code\n\n"),
            None
        );

        // 鍙湁鍓嶉潰鏈夊弽寮曞彿
        assert_eq!(find_real_thinking_end_tag("`</thinking>\n\n"), None);

        // 鍙湁鍚庨潰鏈夊弽寮曞彿
        assert_eq!(find_real_thinking_end_tag("</thinking>`\n\n"), None);
    }

    #[test]
    fn test_find_real_thinking_end_tag_with_quotes() {
        // 琚弻寮曞彿鍖呰９鐨勫簲璇ヨ璺宠繃
        assert_eq!(find_real_thinking_end_tag("\"</thinking>\"\n\n"), None);
        assert_eq!(
            find_real_thinking_end_tag("the string \"</thinking>\" is a tag\n\n"),
            None
        );

        // 琚崟寮曞彿鍖呰９鐨勫簲璇ヨ璺宠繃
        assert_eq!(find_real_thinking_end_tag("'</thinking>'\n\n"), None);
        assert_eq!(
            find_real_thinking_end_tag("use '</thinking>' as marker\n\n"),
            None
        );

        // 娣峰悎鎯呭喌锛氬弻寮曞彿鍖呰９鍚庢湁鐪熸鐨勬爣绛?
        assert_eq!(
            find_real_thinking_end_tag("about \"</thinking>\" tag</thinking>\n\n"),
            Some(23)
        );

        // 娣峰悎鎯呭喌锛氬崟寮曞彿鍖呰９鍚庢湁鐪熸鐨勬爣绛?
        assert_eq!(
            find_real_thinking_end_tag("about '</thinking>' tag</thinking>\n\n"),
            Some(23)
        );
    }

    #[test]
    fn test_find_real_thinking_end_tag_mixed() {
        // 鍏堟湁琚寘瑁圭殑锛屽悗鏈夌湡姝ｇ殑缁撴潫鏍囩
        assert_eq!(
            find_real_thinking_end_tag("discussing `</thinking>` tag</thinking>\n\n"),
            Some(28)
        );

        // 澶氫釜琚寘瑁圭殑锛屾渶鍚庝竴涓槸鐪熸鐨?
        assert_eq!(
            find_real_thinking_end_tag("`</thinking>` and `</thinking>` done</thinking>\n\n"),
            Some(36)
        );

        // 澶氱寮曠敤瀛楃娣峰悎
        assert_eq!(
            find_real_thinking_end_tag(
                "`</thinking>` and \"</thinking>\" and '</thinking>' done</thinking>\n\n"
            ),
            Some(54)
        );
    }

    #[test]
    fn test_tool_use_immediately_after_thinking_filters_end_tag_and_closes_thinking_block() {
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all_events = Vec::new();

        // thinking 鍐呭浠?`</thinking>` 缁撳熬锛屼絾鍚庨潰娌℃湁 `\n\n`锛堟ā鎷熺揣璺?tool_use 鐨勫満鏅級
        all_events.extend(ctx.process_assistant_response("<thinking>abc</thinking>"));

        let tool_events = ctx.process_tool_use(&crate::kiro::model::events::ToolUseEvent {
            name: "Write".to_string(),
            tool_use_id: "tool_1".to_string(),
            input: "{}".to_string(),
            stop: false,
        });
        all_events.extend(tool_events);

        all_events.extend(ctx.generate_final_events());

        // 涓嶅簲鎶?`</thinking>` 褰撲綔 thinking 鍐呭杈撳嚭
        assert!(
            all_events.iter().all(|e| {
                !(e.event == "content_block_delta"
                    && e.data["delta"]["type"] == "thinking_delta"
                    && e.data["delta"]["thinking"] == "</thinking>")
            }),
            "`</thinking>` should be filtered from output"
        );

        // thinking block 蹇呴』鍦?tool_use block 涔嬪墠鍏抽棴
        let thinking_index = ctx
            .thinking_block_index
            .expect("thinking block index should exist");
        let pos_thinking_stop = all_events.iter().position(|e| {
            e.event == "content_block_stop"
                && e.data["index"].as_i64() == Some(thinking_index as i64)
        });
        let pos_tool_start = all_events.iter().position(|e| {
            e.event == "content_block_start" && e.data["content_block"]["type"] == "tool_use"
        });
        assert!(
            pos_thinking_stop.is_some(),
            "thinking block should be stopped"
        );
        assert!(pos_tool_start.is_some(), "tool_use block should be started");
        assert!(
            pos_thinking_stop.unwrap() < pos_tool_start.unwrap(),
            "thinking block should stop before tool_use block starts"
        );
    }

    #[test]
    fn test_final_flush_filters_standalone_thinking_end_tag() {
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all_events = Vec::new();
        all_events.extend(ctx.process_assistant_response("<thinking>abc</thinking>"));
        all_events.extend(ctx.generate_final_events());

        assert!(
            all_events.iter().all(|e| {
                !(e.event == "content_block_delta"
                    && e.data["delta"]["type"] == "thinking_delta"
                    && e.data["delta"]["thinking"] == "</thinking>")
            }),
            "`</thinking>` should be filtered during final flush"
        );
    }

    #[test]
    fn test_thinking_strips_leading_newline_same_chunk() {
        // <thinking>\n 鍦ㄥ悓涓€涓?chunk 涓紝\n 搴旇鍓ョ
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let events = ctx.process_assistant_response("<thinking>\nHello world");

        // 鎵惧埌鎵€鏈?thinking_delta 浜嬩欢
        let thinking_deltas: Vec<_> = events
            .iter()
            .filter(|e| {
                e.event == "content_block_delta" && e.data["delta"]["type"] == "thinking_delta"
            })
            .collect();

        // 鎷兼帴鎵€鏈?thinking 鍐呭
        let full_thinking: String = thinking_deltas
            .iter()
            .map(|e| e.data["delta"]["thinking"].as_str().unwrap_or(""))
            .collect();

        assert!(
            !full_thinking.starts_with('\n'),
            "thinking content should not start with \\n, got: {:?}",
            full_thinking
        );
    }

    #[test]
    fn test_thinking_strips_leading_newline_cross_chunk() {
        // <thinking> 鍦ㄧ涓€涓?chunk 鏈熬锛孿n 鍦ㄧ浜屼釜 chunk 寮€澶?
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let events1 = ctx.process_assistant_response("<thinking>");
        let events2 = ctx.process_assistant_response("\nHello world");

        let mut all_events = Vec::new();
        all_events.extend(events1);
        all_events.extend(events2);

        let thinking_deltas: Vec<_> = all_events
            .iter()
            .filter(|e| {
                e.event == "content_block_delta" && e.data["delta"]["type"] == "thinking_delta"
            })
            .collect();

        let full_thinking: String = thinking_deltas
            .iter()
            .map(|e| e.data["delta"]["thinking"].as_str().unwrap_or(""))
            .collect();

        assert!(
            !full_thinking.starts_with('\n'),
            "thinking content should not start with \\n across chunks, got: {:?}",
            full_thinking
        );
    }

    #[test]
    fn test_thinking_no_strip_when_no_leading_newline() {
        // <thinking> 鍚庣洿鎺ヨ窡鍐呭锛堟棤 \n锛夛紝鍐呭搴斿畬鏁翠繚鐣?
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let events = ctx.process_assistant_response("<thinking>abc</thinking>\n\ntext");

        let thinking_deltas: Vec<_> = events
            .iter()
            .filter(|e| {
                e.event == "content_block_delta" && e.data["delta"]["type"] == "thinking_delta"
            })
            .collect();

        let full_thinking: String = thinking_deltas
            .iter()
            .filter(|e| {
                !e.data["delta"]["thinking"]
                    .as_str()
                    .unwrap_or("")
                    .is_empty()
            })
            .map(|e| e.data["delta"]["thinking"].as_str().unwrap_or(""))
            .collect();

        assert_eq!(full_thinking, "abc", "thinking content should be 'abc'");
    }

    #[test]
    fn test_text_after_thinking_strips_leading_newlines() {
        // `</thinking>\n\n` 鍚庣殑鏂囨湰涓嶅簲浠?\n\n 寮€澶?
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let events = ctx.process_assistant_response("<thinking>\nabc</thinking>\n\n浣犲ソ");

        let text_deltas: Vec<_> = events
            .iter()
            .filter(|e| e.event == "content_block_delta" && e.data["delta"]["type"] == "text_delta")
            .collect();

        let full_text: String = text_deltas
            .iter()
            .map(|e| e.data["delta"]["text"].as_str().unwrap_or(""))
            .collect();

        assert!(
            !full_text.starts_with('\n'),
            "text after thinking should not start with \\n, got: {:?}",
            full_text
        );
        assert_eq!(full_text, "浣犲ソ");
    }

    /// 杈呭姪鍑芥暟锛氫粠浜嬩欢鍒楄〃涓彁鍙栨墍鏈?thinking_delta 鐨勬嫾鎺ュ唴瀹?
    fn collect_thinking_content(events: &[SseEvent]) -> String {
        events
            .iter()
            .filter(|e| {
                e.event == "content_block_delta" && e.data["delta"]["type"] == "thinking_delta"
            })
            .map(|e| e.data["delta"]["thinking"].as_str().unwrap_or(""))
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 杈呭姪鍑芥暟锛氫粠浜嬩欢鍒楄〃涓彁鍙栨墍鏈?text_delta 鐨勬嫾鎺ュ唴瀹?
    fn collect_text_content(events: &[SseEvent]) -> String {
        events
            .iter()
            .filter(|e| e.event == "content_block_delta" && e.data["delta"]["type"] == "text_delta")
            .map(|e| e.data["delta"]["text"].as_str().unwrap_or(""))
            .collect()
    }

    #[test]
    fn test_end_tag_newlines_split_across_events() {
        // `</thinking>\n` 鍦?chunk 1锛宍\n` 鍦?chunk 2锛宍text` 鍦?chunk 3
        // 纭繚 `</thinking>` 涓嶄細琚儴鍒嗗綋浣?thinking 鍐呭鍙戝嚭
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all = Vec::new();
        all.extend(ctx.process_assistant_response("<thinking>\nabc</thinking>\n"));
        all.extend(ctx.process_assistant_response("\n"));
        all.extend(ctx.process_assistant_response("浣犲ソ"));
        all.extend(ctx.generate_final_events());

        let thinking = collect_thinking_content(&all);
        assert_eq!(
            thinking, "abc",
            "thinking should be 'abc', got: {:?}",
            thinking
        );

        let text = collect_text_content(&all);
        assert_eq!(text, "浣犲ソ", "text should be '浣犲ソ', got: {:?}", text);
    }

    #[test]
    fn test_end_tag_alone_in_chunk_then_newlines_in_next() {
        // `</thinking>` 鍗曠嫭鍦ㄤ竴涓?chunk锛宍\n\ntext` 鍦ㄤ笅涓€涓?chunk
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all = Vec::new();
        all.extend(ctx.process_assistant_response("<thinking>\nabc</thinking>"));
        all.extend(ctx.process_assistant_response("\n\n浣犲ソ"));
        all.extend(ctx.generate_final_events());

        let thinking = collect_thinking_content(&all);
        assert_eq!(
            thinking, "abc",
            "thinking should be 'abc', got: {:?}",
            thinking
        );

        let text = collect_text_content(&all);
        assert_eq!(text, "浣犲ソ", "text should be '浣犲ソ', got: {:?}", text);
    }

    #[test]
    fn test_start_tag_newline_split_across_events() {
        // `\n\n` 鍦?chunk 1锛宍<thinking>` 鍦?chunk 2锛宍\n` 鍦?chunk 3
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all = Vec::new();
        all.extend(ctx.process_assistant_response("\n\n"));
        all.extend(ctx.process_assistant_response("<thinking>"));
        all.extend(ctx.process_assistant_response("\n"));
        all.extend(ctx.process_assistant_response("abc</thinking>\n\ntext"));
        all.extend(ctx.generate_final_events());

        let thinking = collect_thinking_content(&all);
        assert_eq!(
            thinking, "abc",
            "thinking should be 'abc', got: {:?}",
            thinking
        );

        let text = collect_text_content(&all);
        assert_eq!(text, "text", "text should be 'text', got: {:?}", text);
    }

    #[test]
    fn test_full_flow_maximally_split() {
        // 鏋佺鎷嗗垎锛氭瘡涓叧閿竟鐣岄兘鍦ㄤ笉鍚?chunk
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all = Vec::new();
        // \n\n<thinking>\n 鎷嗘垚澶氭
        all.extend(ctx.process_assistant_response("\n"));
        all.extend(ctx.process_assistant_response("\n"));
        all.extend(ctx.process_assistant_response("<thin"));
        all.extend(ctx.process_assistant_response("king>"));
        all.extend(ctx.process_assistant_response("\n"));
        all.extend(ctx.process_assistant_response("hello"));
        // </thinking>\n\n 鎷嗘垚澶氭
        all.extend(ctx.process_assistant_response("</thi"));
        all.extend(ctx.process_assistant_response("nking>"));
        all.extend(ctx.process_assistant_response("\n"));
        all.extend(ctx.process_assistant_response("\n"));
        all.extend(ctx.process_assistant_response("world"));
        all.extend(ctx.generate_final_events());

        let thinking = collect_thinking_content(&all);
        assert_eq!(
            thinking, "hello",
            "thinking should be 'hello', got: {:?}",
            thinking
        );

        let text = collect_text_content(&all);
        assert_eq!(text, "world", "text should be 'world', got: {:?}", text);
    }

    #[test]
    fn test_thinking_only_sets_max_tokens_stop_reason() {
        // 鏁翠釜娴佸彧鏈?thinking 鍧楋紝娌℃湁 text 涔熸病鏈?tool_use锛宻top_reason 搴斾负 max_tokens
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all_events = Vec::new();
        all_events.extend(ctx.process_assistant_response("<thinking>\nabc</thinking>"));
        all_events.extend(ctx.generate_final_events());

        let message_delta = all_events
            .iter()
            .find(|e| e.event == "message_delta")
            .expect("should have message_delta event");

        assert_eq!(
            message_delta.data["delta"]["stop_reason"], "max_tokens",
            "stop_reason should be max_tokens when only thinking is produced"
        );

        // 搴旇ˉ鍙戜竴濂楀畬鏁寸殑 text 浜嬩欢锛坈ontent_block_start + delta 绌烘牸 + content_block_stop锛?
        assert!(
            all_events.iter().any(|e| {
                e.event == "content_block_start" && e.data["content_block"]["type"] == "text"
            }),
            "should emit text content_block_start"
        );
        assert!(
            all_events.iter().any(|e| {
                e.event == "content_block_delta"
                    && e.data["delta"]["type"] == "text_delta"
                    && e.data["delta"]["text"] == " "
            }),
            "should emit text_delta with a single space"
        );
        // text block 搴旇 generate_final_events 鑷姩鍏抽棴
        let text_block_index = all_events
            .iter()
            .find_map(|e| {
                if e.event == "content_block_start" && e.data["content_block"]["type"] == "text" {
                    e.data["index"].as_i64()
                } else {
                    None
                }
            })
            .expect("text block should exist");
        assert!(
            all_events.iter().any(|e| {
                e.event == "content_block_stop"
                    && e.data["index"].as_i64() == Some(text_block_index)
            }),
            "text block should be stopped"
        );
    }

    #[test]
    fn test_thinking_with_text_keeps_end_turn_stop_reason() {
        // thinking + text 鐨勬儏鍐碉紝stop_reason 搴斾负 end_turn
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all_events = Vec::new();
        all_events.extend(ctx.process_assistant_response("<thinking>\nabc</thinking>\n\nHello"));
        all_events.extend(ctx.generate_final_events());

        let message_delta = all_events
            .iter()
            .find(|e| e.event == "message_delta")
            .expect("should have message_delta event");

        assert_eq!(
            message_delta.data["delta"]["stop_reason"], "end_turn",
            "stop_reason should be end_turn when text is also produced"
        );
    }

    #[test]
    fn test_thinking_with_tool_use_keeps_tool_use_stop_reason() {
        // thinking + tool_use 鐨勬儏鍐碉紝stop_reason 搴斾负 tool_use
        let mut ctx =
            StreamContext::new_with_thinking("test-model", 200_000, 1, true, HashMap::new());
        let _initial_events = ctx.generate_initial_events();

        let mut all_events = Vec::new();
        all_events.extend(ctx.process_assistant_response("<thinking>\nabc</thinking>"));
        all_events.extend(
            ctx.process_tool_use(&crate::kiro::model::events::ToolUseEvent {
                name: "test_tool".to_string(),
                tool_use_id: "tool_1".to_string(),
                input: "{}".to_string(),
                stop: true,
            }),
        );
        all_events.extend(ctx.generate_final_events());

        let message_delta = all_events
            .iter()
            .find(|e| e.event == "message_delta")
            .expect("should have message_delta event");

        assert_eq!(
            message_delta.data["delta"]["stop_reason"], "tool_use",
            "stop_reason should be tool_use when tool_use is present"
        );
    }
}
