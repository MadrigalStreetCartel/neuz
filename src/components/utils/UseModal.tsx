import { useState } from "react";

const useModal = (parent: { isShown: boolean; close: () => void; open: () => void; } | undefined = undefined) => {
  const [isShown, setIsShown] = useState(false);

  function toggle() {
    if(isShown) {
        close()
    } else {
        open()
    }
  }

  function open() {
    if (parent?.isShown) {
        parent.close()
    }
    setIsShown(true);
  }

  function close() {
    if (parent?.isShown == false) {
        parent.open()
    }
    setIsShown(false);
  }

  return {
    isShown: isShown,
    toggle,
    open,
    close
  };
};

export default useModal;
