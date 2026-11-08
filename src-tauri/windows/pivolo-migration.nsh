; Migrate the previous Shot-VILLA NSIS install in place when productName is
; Pivolo. Keeping the existing directory lets the updater replace the running
; app instead of leaving a second installation beside it.
!macro NSIS_HOOK_PREINSTALL
  ReadRegStr $R0 HKCU "Software\Jascha Coenen\Shot-VILLA" ""
  StrCmp $R0 "" pivolo_migration_no_legacy_install
  IfFileExists "$R0\shot-villa.exe" 0 pivolo_migration_no_legacy_install
  StrCpy $INSTDIR "$R0"
  pivolo_migration_no_legacy_install:
!macroend

; The files are now installed at the previous path and the Pivolo shortcut and
; registry entries have been created. Remove only the old-name metadata and
; shortcuts; retain the app data under the unchanged bundle identifier.
!macro NSIS_HOOK_POSTINSTALL
  Delete "$SMPROGRAMS\Shot-VILLA.lnk"
  Delete "$DESKTOP\Shot-VILLA.lnk"
  DeleteRegKey HKCU "Software\Jascha Coenen\Shot-VILLA"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Shot-VILLA"
!macroend
