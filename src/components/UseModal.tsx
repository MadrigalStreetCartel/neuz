import { useState } from "react";

const useModal = () => {
  const [isShowing, setIsShowing] = useState(false);

  function toggle() {
    setIsShowing(!isShowing);
  }

  function open() {
    setIsShowing(true);
  }

  function close() {
    setIsShowing(false);
  }

  return {
    isShowing,
    toggle,
    open,
    close
  };
};

export default useModal;
