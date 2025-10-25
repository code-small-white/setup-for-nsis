import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';

let mainExe = ''

function App() {
  const [installDir, setInstallDir] = useState('')
  const [currentExePath, setCurrentExePath] = useState('')
  const [startCmd, setStartCmd] = useState('')
  const [doFinish, setDoFinish] = useState(false)
  const installDirWithModo = installDir.endsWith('\\Modo Manager') ? installDir : installDir + '\\Modo Manager'
  console.warn({ installDirWithModo })
  const isUninstall = (currentExePath.split('\\').pop() || '').toLowerCase().startsWith('uninstall')

  useEffect(() => {
    getCurrentWindow().show()
    invoke<string>('get_default_install_dir').then(setInstallDir)
    invoke<string>('current_exe_path').then(setCurrentExePath)
    invoke<string>('start_cmd').then(setStartCmd)
  }, [])

  const choosePath = async () => {
    // Open a dialog
    const selectedInstallDir = await open({
      multiple: false,
      directory: true,
      defaultPath: installDir
    });
    selectedInstallDir && setInstallDir(selectedInstallDir)
    console.log(selectedInstallDir, invoke('check_path_exists', { path: selectedInstallDir }).then(console.log), invoke('check_path_exists', { path: selectedInstallDir + '\\aa' }).then(console.warn));
    // Prints target_dir path and name to the console
  }

  const install = async () => {
    console.warn('执行', installDir)
    setStartCmd('提取文件。。。')
    let i = 0
    if (!installDir) return
    await invoke('delete_nsis_log').then(console.warn)
    const t = setInterval(() => {
      i++
      setStartCmd('提取文件。。。' + i)
    }, 990);
    const setupExe = await invoke('release_main_setup_exe')
    clearInterval(t)

    console.warn('执行', installDir, { setupExe })
    invoke('ps_exe', { file: setupExe, args: ['/S', `/D="${installDirWithModo}"`] })
    let uninstallExe = ''
    while (!mainExe) {
      console.count('while')
      await new Promise<void>(resolve => {
        setTimeout(async () => {
          i++
          setStartCmd('安装中。。。' + i)
          console.log(installDirWithModo + '\\modo-nsis.log')
          await invoke<string>('read_nsis_log').then(log => {
            const logObj = Object.fromEntries(log.split('\r').map(it => it.trim().split('=')))
            console.log(log, logObj)
              ; ({ mainExe, uninstallExe } = logObj)
            console.warn({ mainExe, uninstallExe })

          }).catch(e => console.warn(e))
            .finally(resolve)
        }, 1000);
      })
    }
    // TODO 清理临时文件
    console.warn({ mainExe })
    invoke('check_path_exists', { path: [installDirWithModo, `Uninstall ${mainExe}`].join('\\') }).then(res => {
      console.log([installDirWithModo, `Uninstall ${mainExe}`].join('\\'), res);
    })

    console.error({ install_dir: installDirWithModo, uninstall_exe_file_name: uninstallExe })
    await invoke('uninstallexe_replace', { installDir: installDirWithModo, uninstallExeFileName: uninstallExe })
    setStartCmd('安装。。。finish')
    
    setDoFinish(true)
  }

  const startExe = () => {
    console.warn({ mainExe }, 'starexe', [installDirWithModo, mainExe].join("\\"))
    mainExe && invoke('run_exe', { path: [installDirWithModo, mainExe].join("\\") })
  }

  const uninstallExe = () => {
    const setupExe = currentExePath.split('\\')
    setupExe.splice(-1, 1, 'Real Uninstall.exe')
    console.log(setupExe.join('\\'))
    invoke('ps_exe', { file: setupExe, args: ['/S'] })
  }


  return (
    <main className="container">
      <h4>currentExePath:{currentExePath}</h4>
      <h4>startCmd:{startCmd}</h4>
      {isUninstall ?
        <button onClick={uninstallExe}>卸载</button> :
        <>
          <h5>installDirWithModo:{installDirWithModo}</h5>
          <button onClick={choosePath}>选择安装路径</button>
          <button onClick={install}>安装</button>
          {doFinish && <button onClick={startExe}>启动</button>}
        </>}
    </main>
  );
}

export default App;
