#include <fstream>
#include <string>
#include <string_view>
#include <optional>
#include <chrono>

#include "log.hpp"
#include "../dobby/dobby.h"


void(*ori_postComposition)() = nullptr; // 保存原函数的指针
long long __hook_frametime = 0; // 缓存帧间隔时间
auto __hook_clock = std::chrono::steady_clock::now(); // 保存时间戳

// 从文件读取可用的hook位点(由shell通过readelf扫描symbol提供)
// shell应该保证symbol存在
static std::optional<std::string> read_symbol() {
    std::ifstream symbol_file = std::ifstream("/sdcard/Android/surfaceflinger/available_symbol");
    
    if(!symbol_file.is_open())
        return std::nullopt;
    
    std::string symbol = std::string();
    symbol_file >> symbol;
    
    if(!symbol.empty())
        return std::optional<std::string>(symbol);
    else
        return std::nullopt;
}

void postComposition_hooked() {
    ori_postComposition(); // 调用原来的函数

    auto now = std::chrono::steady_clock::now();
    auto duration = now - __hook_clock;
    __hook_frametime = std::chrono::duration_cast<std::chrono::nanoseconds>(duration).count();
    __hook_clock = now;

    LOGD("Get __hook_frametime %lld ns", __hook_frametime);
    // todo: 建立某种ipc通信传递__hook_frametime
}

// 功能需要被dlsym调用出来，传入的应该是函数的symbol
// 需要用__attribute__((visibility ("default")))修饰被调用的功能 以确保功能可以被外部调用
// 同时用extern "C"保证以c风格编译(c风格编译，symbols是函数名本身)
extern "C" {
    __attribute__((visibility ("default"))) void hook_surfaceflinger() {
    
        // 获取hook目标symbol
        std::string symbol;
        auto target_symbol = read_symbol();
        if(target_symbol.has_value()) {
            symbol = target_symbol.value();
            LOGD("Try to hook symbol %s", symbol.c_str());
        } else {
            return;
        }
        
        void *sym = DobbySymbolResolver(nullptr, symbol.c_str());
        if(nullptr != sym) {
            LOGD("Finded symbol, hooking");
            DobbyHook(sym, (void*)postComposition_hooked,(void**)&ori_postComposition);
        } else {
            LOGE("Target symbol not found");
        }
    }
}
