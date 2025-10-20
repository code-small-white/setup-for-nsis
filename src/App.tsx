import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';

let setupExe: string | null


function App() {
  const [installDir, setInstallDir] = useState('')
  const [currentExePath, setCurrentExePath] = useState('')
  const [startCmd, setStartCmd] = useState('')
  const installDirWithModo = installDir.endsWith('\\Modo Manager') ? installDir : installDir + '\\Modo Manager'
  console.warn({ installDirWithModo })

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

  const chooseSetup = async () => {
    setupExe = await open({
      multiple: false,
      directory: false,
      filters: [{
        extensions: ['exe'],
        name: ""
      }]
    });
  }
  const install = async () => {
    console.warn('执行', installDir)
    installDir && console.warn('执行', installDir)
    installDir && invoke('ps_exe', { file: setupExe, args: ['/S', `/D="${installDirWithModo}"`] })
    let mainExe = ''
    let uninstallExe = ''
    while (!mainExe) {
      await new Promise<void>(resolve => {
        setTimeout(() => {
          console.log(installDirWithModo + '\\modo-nsis.log')
          invoke<string>('read_nsis_log').then(log => {
            const logObj = Object.fromEntries(log.split('\r').map(it => it.trim().split('=')))
            console.log(log, logObj)
            ;({mainExe, uninstallExe} = logObj)
            console.warn({ mainExe, uninstallExe })

            resolve()
          })
        }, 1000);
      })
    }
    console.warn({ mainExe })
    invoke('check_path_exists', { path: [installDirWithModo, `Uninstall ${mainExe}`].join('\\') }).then(res => {
      console.log([installDirWithModo, `Uninstall ${mainExe}`].join('\\'), res);

    })

    console.error({ install_dir: installDirWithModo, uninstall_exe_file_name: uninstallExe })
    uninstallexeReplace = () => {
      invoke('uninstallexe_replace', { installDir: installDirWithModo, uninstallExeFileName: uninstallExe })
    }
    uninstallexeReplace()
  }

  const startExe = () => {
    invoke<string>('read_nsis_log').then(log => {
      const mainExe = log.split('\r').find(it => it.trim().startsWith('mainExe='))?.split('mainExe=').pop()
      invoke('ps_exe', { file: [installDirWithModo, mainExe].join("\\"), args: [] })
    })

  }
  const uninstallExe = () => {
    installDir && invoke('ps_exe', { file: setupExe, args: ['/S'] })
  }
  let uninstallexeReplace = () => {
    console.log('test')
  }

  return (
    <main className="container">
      <h4>currentExePath:{currentExePath}</h4>
      <h4>startCmd:{startCmd}</h4>
      <h5>installDirWithModo:{installDirWithModo}</h5>
      <button onClick={chooseSetup}>选择setup</button>
      <button onClick={choosePath}>选择安装路径</button>
      <button onClick={install}>安装</button>
      <button onClick={startExe}>启动</button>
      <button onClick={uninstallExe}>卸载</button>
      <button onClick={uninstallexeReplace}>ce测试替换</button>
    </main>
  );
}

export default App;
