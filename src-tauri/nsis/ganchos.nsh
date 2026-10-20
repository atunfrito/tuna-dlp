; Ganchos del instalador de tuna-dlp (ver bundle.windows.nsis en tauri.conf.json).

; Al desinstalar, quitar también el "abrir con windows" si estaba puesto.
!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "tuna-dlp"
!macroend
