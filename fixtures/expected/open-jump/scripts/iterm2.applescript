on run argv
	set target to item 1 of argv
	tell application id "com.googlecode.iterm2"
		set w to 0
		repeat with win in windows
			set w to w + 1
			set t to 0
			repeat with tb in tabs of win
				set t to t + 1
				repeat with s in sessions of tb
					if tty of s is target then
						select win
						tell tb to select
						tell s to select
						activate
						return "window " & w & " tab " & t
					end if
				end repeat
			end repeat
		end repeat
	end tell
	return "not found"
end run
