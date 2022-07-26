local M = {}

local http = require("socket.http")
http.TIMEOUT = 0.1

local imgui = ui_imgui

local connected = true
local old_connected = true
local current_song = {}
local active_device = {}

local volume = imgui.IntPtr(0)
local attempts = 0
local max_attempts = 4

local function get_song()
    local body = http.request("http://localhost:8888/api/v1/current_song")

    if not body then
        attempts = attempts + 1
        return nil
    else
        connected = true
        old_connected = true
        attempts = 0
    end

    return jsonDecode(body)
end

local function get_active_device()
    local body = http.request("http://localhost:8888/api/v1/active_device")

    if not body then
        attempts = attempts + 1
        return nil
    else
        connected = true
        old_connected = true
        attempts = 0
    end

    return jsonDecode(body)
end

local function next_song()
    local url = "http://localhost:8888/api/v1/next_song"
    http.request {
        url = url,
        method = "POST",
    }
end

local function previous_song()
    local url = "http://localhost:8888/api/v1/previous_song"
    http.request {
        url = url,
        method = "POST",
    }
end

local function play_song()
    local body = jsonEncode({
        offset = {
            position = 0
        },
        position_ms = 0
    })

    local url = "http://localhost:8888/api/v1/play_song"
    local respbody = {}
    http.request {
        url = url,
        method = "POST",
        source = ltn12.source.string(body),
        headers = {
            ["Accept"] = "*/*",
            ["Content-Type"] = "application/json",
            ["Content-Length"] = #body,
            ["User-Agent"] = "BeamNG",
            ["Connection"] = "keep-alive",
            ["Accept-Encoding"] = "gzip, deflate, br"
        },
        sink = ltn12.sink.table(respbody),
    }
end

local function pause_song()
    local url = "http://localhost:8888/api/v1/pause_song"
    http.request {
        url = url,
        method = "POST",
    }
end

local function seek(time_ms)
    local url = "http://localhost:8888/api/v1/seek/" .. time_ms
    http.request {
        url = url,
        method = "POST",
    }
end

local function set_volume(volume)
    if volume < 0 or volume > 100 then
        log("E", "set_volume", "invalid volume, must be between 0 and 100")
        return
    end

    local url = "http://localhost:8888/api/v1/volume/" .. tostring(volume)
    http.request {
        url = url,
        method = "POST",
    }
end

local lastUpdate = 0
local volumeUpdate = 0
local volumeChanged = false

local function onUpdate()
    if attempts >= max_attempts then
        connected = false
        current_song = {}
        active_device = {}

        if not connected and old_connected then
            log("E", "onUpdate", "failed to connect to spotify")
            old_connected = false
        end
    end
    
    if not connected then return end

    local now = os.clock()
    if os.clock() - lastUpdate > 0.1 then
        lastUpdate = now
        current_song = get_song()
        active_device = get_active_device()
    end

    if volumeChanged then
        if os.clock() - volumeUpdate > 1.6 then
            volumeUpdate = now
            if active_device and active_device.device then
                volume = imgui.IntPtr(active_device.device.volume_percent)
            else
                log("W", "get_active_device", "failed to get active device")
            end

            volumeChanged = false
        end
    else
        if os.clock() - volumeUpdate > 0.1 then
            volumeUpdate = now
            if active_device and active_device.device then
                volume = imgui.IntPtr(active_device.device.volume_percent)
            else
                log("W", "get_active_device", "failed to get active device")
            end
        end
    end

    if imgui.Begin("Spotify Controller") then
        local song = current_song
        if not song or song and not song.item then
            imgui.End()
            return
        end

        local song_name = song.item.name or "None"

        imgui.Text("Song: " .. song_name)

        if imgui.Button("Previous") then
            previous_song()
        end
        imgui.SameLine()
        if imgui.Button("Next") then
            next_song()
        end

        -- progress bar
        local time_minutes = math.floor(song.progress_ms / 60000)
        local time_seconds = math.floor((song.progress_ms % 60000) / 1000)
        local time_str = string.format("%02d:%02d", time_minutes, time_seconds)
        local time_ms = song.progress_ms
        local duration_ms = song.item.duration_ms
        local progress = time_ms / duration_ms
        imgui.ProgressBar(progress, imgui.ImVec2(0.0, 0.0), time_str)
        if imgui.IsItemHovered() and imgui.IsMouseDown(0) then
            local mouse_x = imgui.GetMousePos().x
            local progress_x = imgui.GetItemRectMin().x
            local progress_width = imgui.GetItemRectSize().x
            local time_ms = math.floor((mouse_x - progress_x) / progress_width * duration_ms)
            seek(time_ms)
        end

        imgui.SameLine()
        if song.is_playing then
            if imgui.Button("Pause") then
                pause_song()
            end
        else
            if imgui.Button("Play") then
                play_song()
            end
        end

        -- volume
        if active_device then
            if imgui.SliderInt("Volume", volume, 0, 100) then
                volumeChanged = true
                set_volume(volume[0])
            end
        end

        imgui.End()
    end
end

local function reconnect()
    attempts = 0
    connected = true
    old_connected = true
end

local function is_connected()
    return connected
end

M.onUpdate = onUpdate
M.reconnect = reconnect
M.is_connected = is_connected

M.get_song = get_song
M.get_active_device = get_active_device

M.next_song = next_song
M.previous_song = previous_song
M.play_song = play_song
M.pause_song = pause_song
M.seek = seek
M.set_volume = set_volume

return M