export class PeakingFilter  {

    constructor() {}

    public computeResponseAtFrequency(frequency: any, filterFrequency, filterBoost, filterQ) {

        if(!frequency) {
          throw new Error("Deve essere impèutata una frequenza");
        }

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

        const denominator = Math.pow((1 - Math.pow(filterCoeff, 2)), 2) + Math.pow((filterCoeff * (1 / (A * filterQ))), 2);
        const realPart = Math.pow((filterCoeff / filterQ), 2) + Math.pow((1 - Math.pow(filterCoeff, 2)), 2);
        const imaginaryPart = (1 - Math.pow(filterCoeff, 2)) * (((A * filterCoeff) / filterQ) - (filterCoeff / (A * filterQ)));
        filterModule = 20 * Math.log10(Math.sqrt(Math.pow(realPart / denominator, 2) + Math.pow(imaginaryPart / denominator, 2)));

        return filterModule;
      }
}
