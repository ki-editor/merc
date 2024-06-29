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
        padding: 32,
      }}
    >
      <div
        style={{
          color: "#007a80",
          fontSize: "min(20vw, 120px)",
          letterSpacing: 32,
          paddingLeft: 32,
        }}
      >
        MERC
      </div>
      <div style={{ fontSize: "min(5vw, 36px)" }}>
        The MErcilessly Redundant Config language
      </div>
    </div>
  );
};
