//! Valid until 2100-01-01 AOSP RSA-2048 placeholder certificate block.
//! Tested and verified with PKCS#1 v1.5 SHA256withRSA signature chain.

pub const AOSP_RSA_2100_XML: &str = r#"        <Key algorithm="rsa">
            <PrivateKey format="pem">
-----BEGIN RSA PRIVATE KEY-----
MIICXQIBAAKBgQDAgyPcVogbuDAgafWwhWHG7r5/BeL1qEIEir6LR752/q7yXPKb
KvoyABQWAUKZiaFfz8aBXrNjWDwv0vIL5Jgyg92BSxbX4YVBeuVKvClqOm21wAQI
O2jFVsHwIzmRZBmGTVC3TUCuykhMdzVsiVoMJ1q/rEmdXX0jYvKcXgLocQIDAQAB
AoGBAL6GCwuZqAKm+xpZQ4p7txUGWwmjbcbpysxr88AsNNfXnpTGYGQo2Ix7f2V3
wc3qZAdKvo5yht8fCBHclygmCGjeldMu/Ja20IT/JxpfYN78xwPno45uKbqaPF/C
woB2tqiWrx0014gozpvdsfNPnJQEQweBKY4gExZyW728mTpBAkEA4cbZJ2RsCRbs
NoJtWUmDdAwh8bB0xKGlmGfGaXlchdPcRkxbkp6Uv7NODcxQFLEPEzQat/3V9gQU
0qMmytQcxQJBANpIWZd4XNVjD7D9jFJU+Y5TjhiYOq6ea35qWntdNDdVuSGOvUAy
DSg4fXifdvohi8wti2il9kGPu+ylF5qzr70CQFD+/DJklVlhbtZTThVFCTKdk6PY
ENvlvbmCKSz3i9i624Agro1X9LcdBThv/p6dsnHKNHejSZnbdvjl7OnA1J0CQBW3
TPJ8zv+Ls2vwTZ2DRrCaL3DS9EObDyasfgP36dH3fUuRX9KbKCPwOstdUgDghX/y
qAPpPu6W1iNc6VRCvCECQQCQp0XaiXCyzWSWYDJCKMX4KFb/1mW6moXI1g8bi+5x
fs0scurgHa2GunZU1M9FrbXx8rMdn4Eiz6XxpVcPmy0l
-----END RSA PRIVATE KEY-----
            </PrivateKey>
            <CertificateChain>
                <NumberOfCertificates>2</NumberOfCertificates>
                <Certificate format="pem">
-----BEGIN CERTIFICATE-----
MIICtjCCAh+gAwIBAgICEAAwDQYJKoZIhvcNAQELBQAwYzELMAkGA1UEBhMCVVMx
EzARBgNVBAgMCkNhbGlmb3JuaWExFjAUBgNVBAcMDU1vdW50YWluIFZpZXcxFTAT
BgNVBAoMDEdvb2dsZSwgSW5jLjEQMA4GA1UECwwHQW5kcm9pZDAeFw0xNjAxMDEw
MDAwMDBaFw0xMDAxMDEwMDAwMDBaMHYxCzAJBgNVBAYTAlVTMRMwEQYDVQQIDApD
YWxpZm9ybmlhMRUwEwYDVQQKDAxHb29nbGUsIEluYy4xEDAOBgNVBAsMB0FuZHJv
aWQxKTAnBgNVBAMMIEFuZHJvaWQgU29mdHdhcmUgQXR0ZXN0YXRpb24gS2V5MIGf
MA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDAgyPcVogbuDAgafWwhWHG7r5/BeL1
qEIEir6LR752/q7yXPKbKvoyABQWAUKZiaFfz8aBXrNjWDwv0vIL5Jgyg92BSxbX
4YVBeuVKvClqOm21wAQIO2jFVsHwIzmRZBmGTVC3TUCuykhMdzVsiVoMJ1q/rEmd
XX0jYvKcXgLocQIDAQABo2YwZDAdBgNVHQ4EFgQU1AwQG/jNY7n3OVK1DhNcpteZ
k4YwHwYDVR0jBBgwFoAUKfrxrMxN0kyWQCd1trDpMuUH/i4wEgYDVR0TAQH/BAgw
BgEB/wIBADAOBgNVHQ8BAf8EBAMCAoQwDQYJKoZIhvcNAQELBQADgYEAni1IX4xn
M9waha2Z11Aj6hTsQ7DhnerCI0YecrUZ3GAi5KVoMWwLVcTmnKItnzpPk2sxixZ4
Fg2Iy9mLzICdhPDCJ+NrOPH90ecXcjFZNX2W88V/q52PlmEmT7K+gbsNSQQiis6f
9/VCLiVE+iEHElqDtVWtGIL4QBSbnCBjBH8=
-----END CERTIFICATE-----
                </Certificate>
                <Certificate format="pem">
-----BEGIN CERTIFICATE-----
MIICpzCCAhCgAwIBAgIJAP+U2d2fB8gMMA0GCSqGSIb3DQEBCwUAMGMxCzAJBgNV
BAYTAlVTMRMwEQYDVQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1Nb3VudGFpbiBW
aWV3MRUwEwYDVQQKDAxHb29nbGUsIEluYy4xEDAOBgNVBAsMB0FuZHJvaWQwHhcN
MTYwMTAxMDAwMDAwWhcxMDAxMDEwMDAwMDBaMGMxCzAJBgNVBAYTAlVTMRMwEQYD
VQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1Nb3VudGFpbiBWaWV3MRUwEwYDVQQK
DAxHb29nbGUsIEluYy4xEDAOBgNVBAsMB0FuZHJvaWQxGTAXBgNVBAMTEEdvb2ds
ZSwgSW5jLiwgT1UxMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCia63rbi5E
Ye/VDoLmt5TRdSMfd5tjkWP/96r/C3JHTsAsQ+wzfNes7UA+jCigZtX3hwszl94O
uE4TQKuvpSe/lWmgMdsGUmX4RFlXYfC78hdLt0GAZMAoDo9Sd47b0ke2RekZyOmL
w9vCkT/X11DEHTVm+Vfkl5YLCazOkjWFmwIDAQABo2MwYTAdBgNVHQ4EFgQUKfrx
rMxN0kyWQCd1trDpMuUH/i4wHwYDVR0jBBgwFoAUKfrxrMxN0kyWQCd1trDpMuUH
/i4wDwYDVR0TAQH/BAUwAwEB/zAOBgNVHQ8BAf8EBAMCAoQwDQYJKoZIhvcNAQEL
BQADgYEAT3LzNlmNDsG5dFsxWfbwjSVJMJ6jHBwp0kUtILlNX2S06IDHeHqcOd6o
s/W/L3BfRxBcxebrTQaZYdKumgf/93y4q+ucDyQHXrF/unlx/U1bnt8Uqf7f7XzA
iF343ZtkMlbVNZriE/mPzsF83O+kqrJVw4OpLvtc9mL1J1IXvmM=
-----END CERTIFICATE-----
                </Certificate>
            </CertificateChain>
        </Key>
"#;

/// Injects the 2100 RSA block into an ECDSA-only keybox XML before `</Keybox>`.
pub fn inject_rsa_if_missing(keybox_xml: &str) -> String {
    if keybox_xml.contains("algorithm=\"rsa\"") {
        return keybox_xml.to_owned();
    }
    if let Some(pos) = keybox_xml.rfind("</Keybox>") {
        let (head, tail) = keybox_xml.split_at(pos);
        format!("{head}{AOSP_RSA_2100_XML}    {tail}")
    } else {
        keybox_xml.to_owned()
    }
}
