local M = {}

local http = require("socket.http")
http.TIMEOUT = 0.1

local imgui = ui_imgui

local connected = true
local old_connected = true
local current_song = {}
local active_device = {}
local playlists = {}
local tracks = {}
local top_songs = {}
local active_playlist = nil

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

local function play_song(id, uris, pos)
    local body = ""
    if id then
        body = jsonEncode({
            uris = uris,
            offset = {
                position = pos
            },
        })
    else
        body = jsonEncode({
            position_ms = 0
        })
    end

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

local function get_playlists()
    http.TIMEOUT = 5 -- I know, it's a lot.
    local body = http.request("http://localhost:8888/api/v1/playlists")
    http.TIMEOUT = 0.1

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

local function get_tracks(playlist_id)
    http.TIMEOUT = 5 -- I know, it's a lot.
    local body = http.request("http://localhost:8888/api/v1/playlists/" .. playlist_id .. "/tracks")
    http.TIMEOUT = 0.1

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

local function get_top_songs()
    http.TIMEOUT = 5 -- I know, it's a lot.
    local body = http.request("http://localhost:8888/api/v1/top_tracks")
    http.TIMEOUT = 0.1

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

local last_update = 0
local volume_update = 0
local volume_changed = false
local pushed = false

local function draw_playlist()
    if not active_playlist then return end

    local playlist_tracks = tracks[active_playlist]
    for i, track in pairs(playlist_tracks.items) do
        local song_name = track.track.name
        local song_id = track.track.id

        if current_song and current_song.item and song_id == current_song.item.id then
            imgui.PushStyleColor2(imgui.Col_Button, imgui.ImVec4(0.5, 0.5, 0.5, 1))
            pushed = true
        end

        if imgui.Button(song_name, imgui.ImVec2(imgui.GetWindowWidth(), 24)) then
            local all_songs_in_playlist = {}
            for _, song in pairs(playlist_tracks.items) do
                table.insert(all_songs_in_playlist, "spotify:track:" .. song.track.id)
            end

            play_song(track.track.id, all_songs_in_playlist, i-1)
        end

        if pushed then
            imgui.PopStyleColor()
            pushed = false
        end
    end
end

local function draw_top()
    for i, song in pairs(top_songs) do
        local name = song.name
        local id = song.id

        if current_song and current_song.item and id == current_song.item.id then
            imgui.PushStyleColor2(imgui.Col_Button, imgui.ImVec4(0.5, 0.5, 0.5, 1))
            pushed = true
        end

        if imgui.Button(name, imgui.ImVec2(imgui.GetWindowWidth(), 24)) then
            local all_top_songs = {}
            for _, song in pairs(top_songs) do
                table.insert(all_top_songs, "spotify:track:" .. song.id)
            end

            play_song(id, all_top_songs, i-1)
        end

        if pushed then
            imgui.PopStyleColor()
            pushed = false
        end
    end
end

local show_top = false
local can_pop = false

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
    if os.clock() - last_update > 0.1 then
        last_update = now
        current_song = get_song()
        active_device = get_active_device()
    end

    if volume_changed then
        if os.clock() - volume_update > 1.6 then
            volume_update = now
            if active_device and active_device.device then
                volume = imgui.IntPtr(active_device.device.volume_percent)
            else
                log("W", "get_active_device", "failed to get active device")
            end

            volume_changed = false
        end
    else
        if os.clock() - volume_update > 0.1 then
            volume_update = now
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

        local window_width = imgui.GetWindowWidth()
        local window_height = imgui.GetWindowHeight()

        if imgui.BeginChild1("Song Info", imgui.ImVec2(window_width, 128), true) then
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
                    volume_changed = true
                    set_volume(volume[0])
                end
            end

            imgui.EndChild()
        end

        if playlists then
            if imgui.BeginChild1("Playlists", imgui.ImVec2(window_width / 2, window_height - 165), true) then
                if show_top then
                    can_pop = true
                    imgui.PushStyleColor2(imgui.Col_Button, imgui.ImVec4(0.5, 0.5, 0.5, 1))
                end

                if imgui.Button("Top Tracks", imgui.ImVec2(window_width / 2, 24)) then
                    show_top = not show_top
                    if show_top then
                        active_playlist = nil
                    end
                end

                if can_pop then
                    imgui.PopStyleColor()
                    can_pop = false
                end

                for _, playlist in pairs(playlists.items) do
                    if playlist.name ~= "" then
                        local id = playlist.id

                        if active_playlist == id then
                            imgui.PushStyleColor2(imgui.Col_Button, imgui.ImVec4(0.5, 0.5, 0.5, 1))
                            if imgui.Button(playlist.name, imgui.ImVec2(window_width / 2, 24)) then
                                active_playlist = nil
                                show_top = false
                            end
                            imgui.PopStyleColor()
                        else
                            if imgui.Button(playlist.name, imgui.ImVec2(window_width / 2, 24)) then
                                active_playlist = playlist.id
                                show_top = false
                            end
                        end
                    end
                end

                imgui.EndChild()
            end

            imgui.SameLine()
            if imgui.BeginChild1("Playlist", imgui.ImVec2(imgui.GetWindowWidth() / 2, window_height - 165), true) then
                if show_top then
                    draw_top()
                else
                    draw_playlist()
                end
                imgui.EndChild()
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

local function onExtensionLoaded()
    playlists = get_playlists()
    for _, playlist in pairs(playlists.items) do
        tracks[playlist.id] = get_tracks(playlist.id)
    end

    top_songs = get_top_songs().items
end

M.onExtensionLoaded = onExtensionLoaded
M.onUpdate = onUpdate

M.reconnect = reconnect
M.is_connected = is_connected

M.get_playlists = get_playlists
M.get_tracks = get_tracks

M.get_song = get_song
M.get_active_device = get_active_device

M.next_song = next_song
M.previous_song = previous_song
M.play_song = play_song
M.pause_song = pause_song
M.seek = seek
M.set_volume = set_volume

return M