
#[test]
fn canonical_test() {
	let input = "A Α А Ꭺ ᗅ ᴀ ꓮ Ａ 𐊠 𝐀 𝐴 𝑨 𝒜 𝓐 𝔄 𝔸 𝕬 𝖠 𝗔 𝘈 𝘼 𝙰 𝚨 𝛢 𝜜 𝝖 𝞐";
	let normalized = unicode_security::skeleton(input).collect::<String>();
	let result = normalized.to_lowercase(); // convert to lower case
	println!("{}", result);
}
