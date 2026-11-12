on run argv
	set target to item 1 of argv
	tell application "Terminal"
		set w to 0
		repeat with win in windows
			set w to w + 1
			set t to 0
			repeat with tb in tabs of win
				set t to t + 1
				if tty of tb is target then
					set selected tab of win to tb
					set index of win to 1
					activate
					return "window " & w & " tab " & t
				end if
			end repeat
		end repeat
	end tell
	return "not found"
end run
