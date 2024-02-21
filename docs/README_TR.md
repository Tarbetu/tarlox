# Tarbetu's Lox

Lox, [Crafting Interpreters](https://craftinginterpreters.com/) isimli kitapta oluşuturulan dinamik tipli ve yorumlanan bir dildir. Bu dil öğretim amacıyla sade tutulmuştur ve hemen hemen her "imperative" (ya da Sadi Evren Şeker'in çevirisiyle "emirli") dilde var olan özellikler bulunmaktadır. Örneğin:

- Değişken atama
- If/Else
- While ve For döngüsü
- Fonksiyonlar
- Sınıflar ve miras alma

"Tarbetu's Lox" ya da kısaca "Tarlox" ise benim tarafımdan implemente edilmiş bir yorumlayıcıdır. Bu yorumlayıcı değişken atamasını "paralel bir threadde" gerçekleştirmekte, ilgili değişkene erişilmesi gerektiği zaman ana "threadi" kilitlemektedir. Örneğin:

```js
fun func1() {
  # Bir şeyler yapılmakta
  return 666;
};

fun func2(param1) {
  # Bir şeyler daha yapılmakta
  return param1 + 2;
}

var x = func2(func1()); # 1

# Farklı türden işlemler

print is_ready(x); # 2

print x; # 3
```

1 numaralı satırda maliyetli bir değişken ataması yapılmaktadır. x'in değeri farklı bir threadde hesaplanmaktadır.

x'in değeri hesaplanırken önce func1, sonra da func2 ayrı threadlerde tetiklenmiştir ancak func2'nin çalışması için func1'in tamamlanması gerekmemektedir. func2 içerisinde ne zaman func1'in döneceği değer kullanılacaksa o zaman func2'nin döneceği değer beklenecektir.

x'in değeri hesaplanırken program ana akışa devam eder ve "Farklı türden işlemleri" gerçekleştirir. 2 numaralı satırda görüneceği üzere ilgili değerin hesaplanmasının tamamlanıp tamamlamadığı "is_ready" operatörü ile öğrenilebilir. "is_ready" ilgili değere erişmeye çalışmayacak, işleme almayacaktır.

Program ise üçüncü satırda, yani x'e erişildiği anda kilitlenecek ve x'in değerinin hesaplanmasının bekleyecektir.

Eğer değişken atandığı anda değerinin hesaplanması istenirse değişkenler şöyle atanabilir:

```js
await_var x = 5 + 5;
```

x'in değeri hesaplanana kadar ana thread kilitlenecektir.

Bunun dışında argümanlar için paralel akış yine de sağlanacaktır. Eğer akışın değeri hesapladıktan devam etmesini isterseniz parametre gölgelenebilir:

```js
fun func1(params1) {
  await_var params1 = params1;
}
```

Tarlox'ta değişkenler yeniden atanabilir (assignment).

```js
var x = 10;
x = 5;

print x; # 5 yazar
```

Ya da yeni değişken bildirimleri ile gölgelenebilir.

```js
var x = 10;
var x = 20;

print x; # 20 yazar
```


Benim önerim mümkün olduğunca gölgelemeyi kullanmanız olacaktır. Değişkenlerin "değiştirilmesi", değerlerin paralel hesaplamaları esnasında ortaya çıkabilecek çakışmaları tetikleyebilir. Mesela bir threadin bir değişkeni değiştirmeye çalıştığını düşünelim. Yeni değer hesaplandıktan sonra kısa bir saniye içerisinde değişken isim alanında erişilemez hâle gelecek, sonra da tekrar yerleştirilecektir. Değer isim alanından çıkartılmışkan başka bir thread erişmeye çalıştığı anda o değişkene erişemeyecektir. Bu da bir çalışma zamanı hatasına sebep olacaktır.

Lox'ta for döngüsündeki değişken nasıl bildirilirse bildirilsin ilgili threadin içerisinde oluşturulur.

```js
  for (var = 0; i < 10; i = i + 1) {
    print i;
  }
```

Eğer gerçek bir paralelizm isteniyorsa (recursion) özyineleme kullanılabilir. Parametreler ayrı threadlerde hesaplanacaktır.

```js
fun fib(n) {
  if (n == 0) {
    return 0;
  }
  if (n == 1) {
    return 1;
  }

  return fib(n - 1) + fib(n - 2)
}
```

İlgili örnekte `fib(n - 1)` ve `fib(n - 2)` çağrıları farklı bir threadde hesaplanacaktır. Tarlox'ta özyineleme güvenlidir, stack overflow'a sebep olmaz. Her çağrıda argüman dışında bütün değerler atılır, çağrı içerisinde kaç farklı dal (branch) varsa o kadar argüman tutulur. Aslında fonksiyon çağrısından ziyade fonksiyon sanki bir "goto" ifadesi varmışcasına tekrar tekrar baştan çalıştırılır.

Kişisel önerim hem bilgisayar bilimlerine yatkınlığından ötürü hem de yan etkiler kontrol edilebileceği için özyinelemeyi tercih etmenizdir.

Proje hakkındaki örnek kodları `examples` dizini altında bulabilirsiniz. Ne yazık ki yorum satırları İngilizce.

# Projenin hedefleri

Bu proje bir çeşit hobi projesi, çok az vaktimi ayırabiliyorum ve bu koşullar altında projenin sürdürülebilir olması açısından sade kalmasını istiyorum. Bazı dostlarım, başka dillerde beğenerek kullandıkları özellikleri öneriyorlar. Bu çok doğal bir şey, fakat eklemek istediğiniz özelliğin şu kriterlerini düşünün:

- Sözdizimini karmaşıklaştıracak mı?
- Yan etkiye sebep olacak mı?
- Dilin genel felsefesine ters düşecek mi?

Dilin genel yaklaşımı paralelizmi basit kurallara indirgeyerek dilin temel kullanımı içerisine bunu yedirmek. Mesela dikkat edin, dilde paralelizmi işaret eden bir anahtar kelime yok. Sadece "is_ready" operatörü var ve bu da bana çok doğal geliyor. Tıpkı iki iş yapılırken bir işi yapanın diğer işi yapana "Hazır mı?" demesi gibi. Buna karşı dilin esas doğasında anormali işin paralelde tetiklenmesi değil, sonucun aynı thread'de tetiklenmesi.

Eğer kodunuzda bir yan etki oluşuyorsa, mesela yeniden atama yapıyorsanız ya da çıktınız belli bir sıraya ihtiyaç duyuyorsa `await_var` kullanabilirsiniz.

Her ne kadar yorumlanarak çalışsa da çalışma zamanı maliyetini mümkün olduğunca azaltmak istiyorum. Bu yüzden Erlangvari bir süreç yönetim yapısı yerine işletim sisteminin sunduğu threadleri kullanmak bana makul geliyor. Tarlox programlarının hızlıca çalıştırılabilmesi, bunun için program maliyetinin ek bir yük getirmemesi bana daha uygun geliyor.

Son olarak ileride yapabilirsem bytecode derleyicisi ve bilgisayarlar arası paralelizm de sunmak isterim.
