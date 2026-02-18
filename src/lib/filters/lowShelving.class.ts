export class LowShelvingFilter  {

    constructor() {}

    public computeResponseAtFrequency(frequency, filterFrequency, filterBoost, filterQ) {
        let A = 0;
        let omega0 = 0;
        let filterModule = 0;
        let filterCoeff = 0;
        let exp = 0;
        const denominator1 = 40;

        exp = filterBoost / denominator1;
        A = Math.pow(10, exp);
        omega0 = 2 * Math.PI * filterFrequency;
        filterCoeff = (2 * Math.PI * frequency) / omega0;

        const denominator = Math.pow((1 - A * Math.pow(filterCoeff, 2)), 2) + (A * Math.pow((filterCoeff / filterQ), 2));
        const realPart = A * ((A - Math.pow(filterCoeff, 2)) * (1 - A * Math.pow(filterCoeff, 2)) + A * Math.pow((filterCoeff / filterQ), 2));
        const imaginaryPart = A * filterCoeff * (Math.sqrt(A) / filterQ) * (1 - A) * (1 + Math.pow(filterCoeff, 2));
        filterModule = 20 * Math.log10(Math.sqrt(Math.pow(realPart / denominator, 2) + Math.pow(imaginaryPart / denominator, 2)));

        return filterModule;
      }
}
