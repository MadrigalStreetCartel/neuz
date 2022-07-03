import { RecoilRoot } from "recoil"
import Launcher from "./Launcher"

const App = () => {
  return (
    <div className="App">
      <RecoilRoot>
        <Launcher />
      </RecoilRoot>
    </div>
  )
}

export default App
