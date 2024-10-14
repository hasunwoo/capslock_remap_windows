#Requires AutoHotkey >=2.0- <2.1
#SingleInstance Ignore

g_LastCtrlKeyDownTime := 0
g_AbortSendEsc := false
g_ControlRepeatDetected := false


/*
  IME check for windows 11(newer IME)
  출처: https://github.com/johngrib/simple_vim_guide/blob/master/md/with_korean.md
*/
IME_CHECK(WinTitle) {
    hWnd := WinGetID(WinTitle)
    ; IMC_GETCONVERSIONMODE(0x1)를 사용
    return Send_ImeControl(ImmGetDefaultIMEWnd(hWnd), 0x001, "")
}

Send_ImeControl(DefaultIMEWnd, wParam, lParam) {
    DetectSave := A_DetectHiddenWindows
    DetectHiddenWindows(true)
    result := SendMessage(0x283, wParam, lParam,, "ahk_id " DefaultIMEWnd)
    if DetectSave != A_DetectHiddenWindows
        DetectHiddenWindows(DetectSave)
    return result
}

ImmGetDefaultIMEWnd(hWnd) {
    return DllCall("imm32\ImmGetDefaultIMEWnd", "UInt", hWnd, "UInt")
}

*CapsLock::
{
    global g_LastCtrlKeyDownTime, g_AbortSendEsc, g_ControlRepeatDetected

    if (g_ControlRepeatDetected)
    {
        return
    }

    g_LastCtrlKeyDownTime := A_TickCount
    g_AbortSendEsc := false
    g_ControlRepeatDetected := true

    send("{Ctrl down}")
}


*CapsLock Up::
{
    global g_LastCtrlKeyDownTime, g_AbortSendEsc, g_ControlRepeatDetected

    send("{Ctrl up}")
    g_ControlRepeatDetected := false

    if (g_AbortSendEsc)
    {
        return
    }

    current_time := A_TickCount
    time_elapsed := current_time - g_LastCtrlKeyDownTime
    if (time_elapsed <= 250)
    {
        SendInput("{Esc}")
        if IME_CHECK("A")
            Send("{VK15}")  ; 영문이라면 한영전환 키를 입력해준다.
        }
}

*Esc::
{
    current_state := GetKeyState("CapsLock", "T")
    SetCapsLockState(!current_state)
}

~*^a::
~*^b::
~*^c::
~*^d::
~*^e::
~*^f::
~*^g::
~*^h::
~*^i::
~*^j::
~*^k::
~*^l::
~*^m::
~*^n::
~*^o::
~*^p::
~*^q::
~*^r::
~*^s::
~*^t::
~*^u::
~*^v::
~*^w::
~*^x::
~*^y::
~*^z::
~*^1::
~*^2::
~*^3::
~*^4::
~*^5::
~*^6::
~*^7::
~*^8::
~*^9::
~*^0::
~*^Space::
~*^Backspace::
~*^Delete::
~*^Insert::
~*^Home::
~*^End::
~*^PgUp::
~*^PgDn::
~*^Tab::
~*^Enter::
~*^,::
~*^.::
~*^/::
~*^;::
~*^'::
~*^[::
~*^]::
~*^\::
~*^-::
~*^=::
~*^`::
~*^F1::
~*^F2::
~*^F3::
~*^F4::
~*^F5::
~*^F6::
~*^F7::
~*^F8::
~*^F9::
~*^F10::
~*^F11::
~*^F12::
{
    global g_AbortSendEsc := true
}
