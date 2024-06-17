import * as React from "react";

export const Header = () => {
  return (
    <div
      style={{
        display: "grid",
        placeItems: "center",
        height: "64vh",
        placeContent: "center",
        textAlign: "center",
        gap: 16,
        backgroundColor: "rgb(177,180,255)",
        background:
          "linear-gradient(66deg, rgba(177,180,255,0.5721003134796239) 0%, rgba(124,255,234,0.5203761755485894) 75%)",
      }}
    >
      <div style={{ color: "#007a80", fontSize: 120, letterSpacing: 32 }}>
        MARC
      </div>
      <div style={{ fontSize: 36 }}>
        The MAximally Redundant Config language
      </div>
    </div>
  );
};
