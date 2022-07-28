import React, { DetailedHTMLProps, HTMLAttributes, ReactChild, ReactChildren, ReactNode, useEffect, useRef, useState } from "react";
import styled from "styled-components";
interface IProps {
  open: boolean;
  header?: string | React.ReactNode | null;
  headerClassName?: string;
  titleClassName?: string;
  iconButtonClassName?: string;
  contentClassName?: string;
  contentContainerClassName?: string;
  collapsibleClassName?: string;
  children:React.ReactFragment;
}

const Collapsible: React.FC<IProps> = ({
  open,
  collapsibleClassName = "collapsible-card-edonec",
  headerClassName = "collapsible-header-edonec",
  titleClassName = "title-text-edonec",
  iconButtonClassName = "collapsible-icon-button-edonec",
  contentClassName = "collapsible-content-edonec",
  contentContainerClassName = "collapsible-content-padding-edonec",
  children,
  header
}) => {
  const [height, setHeight] = useState<number | undefined>(
    open ? undefined : 0
  );
  const ref = useRef<HTMLDivElement>(null);
  const handleFilterOpening = () => {
    //setIsOpen((prev) => !prev);
  };
  useEffect(() => {
    if (!height || !open || !ref.current) return undefined;
    // @ts-ignore
    const resizeObserver = new ResizeObserver((el) => {
      setHeight(el[0].contentRect.height);
    });
    resizeObserver.observe(ref.current);
    return () => {
      resizeObserver.disconnect();
    };
  }, [height, open]);
  useEffect(() => {
    if (open) setHeight(ref.current?.getBoundingClientRect().height);
    else setHeight(0);
  }, [open]);

  return (
    <>
      <div className={collapsibleClassName}>
        <div>
            {header &&
                <div className={headerClassName}>

                        <div className={titleClassName}>{header}</div>


                    <button
                    type="button"
                    className={iconButtonClassName}
                    onClick={handleFilterOpening}
                    >
                    <i
                        className={`${
                        open
                            ? "rotate-center-edonec down"
                            : "rotate-center-edonec up"
                        }`}
                    />
                    </button>
                </div>
            }
        </div>
        <div className={contentClassName} style={{ height }}>
          <div ref={ref}>
            <div className={contentContainerClassName}>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
};

export default styled(Collapsible)`

.collapsible-content-edonec {
    overflow: hidden;
    transition: height 0.2s ease-in-out;
}
.title-text-edonec {
    display: block;
    font-size: 1em;
    font-weight: bold;
}
.collapsible-header-edonec {
    display: flex;
    justify-content: space-between;
    padding: 2px 20px 2px 20px;
}
.collapsible-content-padding-edonec {
    display: flex;
    flex-direction: column;
}
.rotate-center-edonec {
    -moz-transition: all 1s linear;
    -webkit-transition: all 1s linear;
    transition: all 0.2s linear;
}
.rotate-center-edonec.down {
    -moz-transform: rotate(180deg);
    -webkit-transform: rotate(180deg);
    transform: rotate(180deg);
}
.rotate-center-edonec.up {
    -moz-transform: rotate(360deg);
    -webkit-transform: rotate(360deg);
    transform: rotate(360deg);
}
.collapsible-icon-button-edonec {
    cursor: pointer;
    background-color: Transparent;
    background-repeat: no-repeat;
    border: none;
    cursor: pointer;
    overflow: hidden;
    outline: none;
}

.collapsible-card-edonec {
    transition: 1s;
    color: white;
}`
